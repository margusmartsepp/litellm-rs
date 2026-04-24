use crate::models::{ResponseMetadata, generated};
use reqwest::header::HeaderMap;

pub struct ResponseHandler;

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UnifiedToolCall {
    pub id: String,
    pub name: String,
    pub arguments: String,
}

#[derive(Debug)]
pub struct UnifiedResponse {
    pub content: String,
    pub tool_calls: Option<Vec<UnifiedToolCall>>,
    pub metadata: ResponseMetadata,
}

impl ResponseHandler {
    /// Parses the raw response and audits the metadata via Bifrost headers.
    pub async fn handle(
        provider: &str,
        headers: &HeaderMap,
        body: &[u8],
        start_time: std::time::Instant,
        status: reqwest::StatusCode,
    ) -> anyhow::Result<UnifiedResponse> {
        let latency = start_time.elapsed();

        // Extract Scientific Audit data from Bifrost Headers
        let cost = headers
            .get("x-litellm-response-cost")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<f64>().ok())
            .unwrap_or(0.0);

        let model_version = headers
            .get("x-litellm-model-id")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string())
            .unwrap_or_else(|| "unknown".to_string());

        let cache_hit = headers
            .get("x-bf-cache-status")
            .and_then(|v| v.to_str().ok())
            .map(|v| v == "hit")
            .unwrap_or(false);

        let failover = headers
            .get("x-bf-failover")
            .and_then(|v| v.to_str().ok())
            .map(|v| v == "true")
            .unwrap_or(false);

        let health_state = headers
            .get("x-bf-health-state")
            .and_then(|v| v.to_str().ok())
            .map(|v| v.to_string());

        let budget_exceeded = status == reqwest::StatusCode::TOO_MANY_REQUESTS ||
            headers.get("x-bf-budget-exceeded").and_then(|v| v.to_str().ok()) == Some("true");

        // Parse body based on provider type
        let mut usage = None;
        let mut id = None;
        let mut finish_reason = None;
        let mut system_fingerprint = None;

        let (content, tool_calls) = if budget_exceeded {
            ("Budget Exceeded".to_string(), None)
        } else if provider == "openai" {
            let res: generated::openai::CreateChatCompletionResponse = serde_json::from_slice(body)?;
            usage = serde_json::to_value(&res.usage).ok();
            id = Some(res.id.clone());
            system_fingerprint = res.system_fingerprint.clone();

            let choice = &res.choices[0];
            finish_reason = Some(serde_json::to_string(&choice.finish_reason)?).map(|s| s.trim_matches('"').to_string());

            let msg = &choice.message;
            let content = msg.content.clone()
                .or_else(|| msg.refusal.clone())
                .unwrap_or_else(|| "".to_string());

            let tool_calls = msg.tool_calls.as_ref().map(|calls| {
                calls.iter().filter_map(|tc| {
                    match tc {
                        generated::openai::ChatCompletionMessageToolCallsItem::ToolCall(call) => {
                            Some(UnifiedToolCall {
                                id: call.id.clone(),
                                name: call.function.name.clone(),
                                arguments: call.function.arguments.clone(),
                            })
                        }
                        _ => None,
                    }
                }).collect()
            });

            (content, tool_calls)
        } else {
            anyhow::bail!("Anthropic currently disabled or unsupported provider: {}", provider)
        };

        Ok(UnifiedResponse {
            content,
            tool_calls,
            metadata: ResponseMetadata {
                cost,
                provider: provider.to_string(),
                latency,
                model_version,
                cache_hit,
                failover,
                health_state,
                budget_exceeded,
                usage,
                id,
                finish_reason,
                system_fingerprint,
            },
        })
    }
}
