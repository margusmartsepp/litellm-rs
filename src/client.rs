use crate::models::LiteLLMRequest;
use crate::translator::Translator;
use crate::response_handler::{ResponseHandler, UnifiedResponse};
use crate::models::generated;
use futures_util::{Stream, StreamExt};
use async_stream::try_stream;

pub struct LiteLLM {
    client: reqwest::Client,
    base_url: String,
    virtual_key: String,
}

impl LiteLLM {
    pub fn new(base_url: &str, virtual_key: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            virtual_key: virtual_key.to_string(),
        }
    }

    pub async fn chat(&self, request: LiteLLMRequest<'_>) -> anyhow::Result<UnifiedResponse> {
        let start_time = std::time::Instant::now();
        let model_lower = request.model.to_lowercase();
        let provider = if model_lower.contains("gpt") || model_lower.contains("openai") || model_lower.contains("gemma") || model_lower.contains("llama") {
            "openai"
        } else {
            "anthropic"
        };

        let mcp_mode = request.mcp_code_mode;
        let extra_headers = request.extra_headers.clone();

        // 1. Translate
        let (payload, endpoint) = Translator::to_provider_payload(request)?;
        let url = format!("{}{}", self.base_url, endpoint);

        // 2. Dispatch with Bifrost & Standard Headers
        let mut request_builder = self.client
            .post(url)
            .header("x-bf-vk", &self.virtual_key)
            .header("Authorization", format!("Bearer {}", self.virtual_key))
            .header("Content-Type", "application/json");

        if mcp_mode {
            request_builder = request_builder.header("x-bf-mcp-code-mode", "true");
        }

        if let Some(extra) = extra_headers {
            for (k, v) in extra {
                request_builder = request_builder.header(k, v);
            }
        }

        let response = request_builder
            .body(payload)
            .send()
            .await?;

        // 3. Handle Response & Audit
        let status = response.status();
        let headers = response.headers().clone();
        let body = response.bytes().await?;

        if !status.is_success() && status != reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Err(anyhow::anyhow!("Upstream Error: {} - {:?}", status, body));
        }

        ResponseHandler::handle(provider, &headers, &body, start_time, status).await
    }

    pub async fn list_models(&self) -> anyhow::Result<Vec<String>> {
        let url = format!("{}/v1/models", self.base_url);
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("List Models Error: {} - {}", status, body);
        }

        let res: generated::openai::ListModelsResponse = response.json().await?;
        Ok(res.data.into_iter().map(|m| m.id).collect())
    }

    pub async fn stream_chat(&self, mut request: LiteLLMRequest<'_>) -> anyhow::Result<impl Stream<Item = anyhow::Result<String>>> {
        request.stream = Some(true);
        let model_lower = request.model.to_lowercase();
        let is_openai = model_lower.contains("gpt") || model_lower.contains("openai") || model_lower.contains("gemma") || model_lower.contains("llama");

        if !is_openai {
            anyhow::bail!("Streaming only supported for OpenAI-compatible providers currently.");
        }

        let mcp_mode = request.mcp_code_mode;
        let extra_headers = request.extra_headers.clone();

        let (payload_json, endpoint) = Translator::to_provider_payload(request)?;

        // Inject stream: true into the JSON payload
        let mut payload: serde_json::Value = serde_json::from_str(&payload_json)?;
        payload["stream"] = serde_json::json!(true);
        let payload = serde_json::to_string(&payload)?;

        let url = format!("{}{}", self.base_url, endpoint);

        let mut request_builder = self.client
            .post(url)
            .header("x-bf-vk", &self.virtual_key)
            .header("Authorization", format!("Bearer {}", self.virtual_key))
            .header("Content-Type", "application/json");

        if mcp_mode {
            request_builder = request_builder.header("x-bf-mcp-code-mode", "true");
        }

        if let Some(extra) = extra_headers {
            for (k, v) in extra {
                request_builder = request_builder.header(k, v);
            }
        }

        let response = request_builder
            .body(payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Stream Error: {} - {}", status, body);
        }

        let mut stream = response.bytes_stream();

        Ok(try_stream! {
            let mut line_buffer = String::new();
            while let Some(chunk_result) = stream.next().await {
                let chunk = chunk_result?;
                let text = String::from_utf8_lossy(&chunk);
                line_buffer.push_str(&text);

                while let Some(line_end) = line_buffer.find('\n') {
                    let line = line_buffer.drain(..line_end + 1).collect::<String>();
                    let line = line.trim();

                    if line.is_empty() { continue; }
                    if line == "data: [DONE]" { break; }

                    if line.starts_with("data: ") {
                        let json_str = &line[6..];
                        if let Ok(value) = serde_json::from_str::<serde_json::Value>(json_str) {
                            if let Some(content) = value.get("choices")
                                .and_then(|c| c.as_array())
                                .and_then(|a| a.first())
                                .and_then(|f| f.get("delta"))
                                .and_then(|d| d.get("content"))
                                .and_then(|v| v.as_str()) {
                                    yield content.to_string();
                            }
                        }
                    }
                }
            }
        })
    }

    pub async fn load_model(&self, model_id: &str) -> anyhow::Result<()> {
        let url = format!("{}/v1/models/load", self.base_url);
        let payload = serde_json::json!({ "model": model_id });
        let response = self.client.post(url).json(&payload).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Load Model Error: {} - {}", status, body);
        }
        Ok(())
    }

    pub async fn unload_model(&self, model_id: &str) -> anyhow::Result<()> {
        let url = format!("{}/v1/models/unload", self.base_url);
        let payload = serde_json::json!({ "model": model_id });
        let response = self.client.post(url).json(&payload).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Unload Model Error: {} - {}", status, body);
        }
        Ok(())
    }

    pub async fn get_download_status(&self) -> anyhow::Result<serde_json::Value> {
        let url = format!("{}/v1/models/download/status", self.base_url);
        let response = self.client.get(url).send().await?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await?;
            anyhow::bail!("Download Status Error: {} - {}", status, body);
        }
        Ok(response.json().await?)
    }
}
