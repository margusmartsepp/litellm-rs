use serde::{Deserialize, Serialize};
use std::borrow::Cow;

pub mod generated {
    pub mod openai {
        include!(concat!(env!("OUT_DIR"), "/openai_types.rs"));
    }
    pub mod anthropic {
        include!(concat!(env!("OUT_DIR"), "/anthropic_types.rs"));
    }
}
/// The Universal Response Metadata for the "Scientific Audit" trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMetadata {
    pub cost: f64,
    pub provider: String,
    pub latency: std::time::Duration,
    pub model_version: String,
    pub cache_hit: bool,
    pub failover: bool,
    pub health_state: Option<String>,
    pub budget_exceeded: bool,
    pub usage: Option<serde_json::Value>,
    pub id: Option<String>,
    pub finish_reason: Option<String>,
    pub system_fingerprint: Option<String>,
}

/// The Unified Request trait that all generated structs must implement.
pub trait UniversalRequest {
    fn to_openai_json(&self) -> anyhow::Result<String>;
    fn model_name(&self) -> &str;
}

/// The main Unified Message struct used by the LiteLLM-rs client.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UnifiedMessage<'a> {
    pub role: Cow<'a, str>,
    pub content: Cow<'a, str>,
    pub images: Option<Vec<Cow<'a, str>>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UnifiedFunction<'a> {
    pub name: Cow<'a, str>,
    pub description: Option<Cow<'a, str>>,
    pub parameters: Option<serde_json::Value>,
    pub strict: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UnifiedTool<'a> {
    #[serde(rename = "type")]
    pub type_: Cow<'a, str>,
    pub function: UnifiedFunction<'a>,
}

/// A zero-copy container for LLM requests.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LiteLLMRequest<'a> {
    pub model: Cow<'a, str>,
    pub messages: Vec<UnifiedMessage<'a>>,
    pub tools: Option<Vec<UnifiedTool<'a>>>,
    pub temperature: Option<f32>,
    pub max_tokens: Option<u32>,
    pub stream: Option<bool>,
    pub mcp_code_mode: bool,
    pub response_format: Option<serde_json::Value>,
    pub border_id: Option<String>,
    pub extra_headers: Option<std::collections::HashMap<String, String>>,
}

/// A zero-copy container for Embedding requests.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LiteLLMEmbeddingRequest<'a> {
    pub model: Cow<'a, str>,
    pub input: Vec<Cow<'a, str>>,
    pub extra_headers: Option<std::collections::HashMap<String, String>>,
}

/// The unified response from an embedding operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteLLMEmbeddingResponse {
    pub data: Vec<UnifiedEmbedding>,
    pub model: String,
    pub usage: serde_json::Value,
    pub metadata: ResponseMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedEmbedding {
    pub embedding: Vec<f64>,
    pub index: i64,
}

// Implementation for OpenAI
impl<'a> From<LiteLLMRequest<'a>> for generated::openai::CreateChatCompletionRequest {
    fn from(req: LiteLLMRequest<'a>) -> Self {
        Self {
            model: generated::openai::ModelIdsShared(req.model.into_owned()),
            messages: req.messages.into_iter().map(|m| {
                match m.role.as_ref() {
                    "system" => generated::openai::ChatCompletionRequestMessage::SystemMessage(
                        generated::openai::ChatCompletionRequestSystemMessage {
                            content: generated::openai::ChatCompletionRequestSystemMessageContent::String(m.content.into_owned()),
                            role: "system".to_string(),
                            name: None,
                        }
                    ),
                    _ => {
                        if m.images.is_none() {
                            generated::openai::ChatCompletionRequestMessage::UserMessage(
                                generated::openai::ChatCompletionRequestUserMessage {
                                    content: generated::openai::ChatCompletionRequestUserMessageContent::String(m.content.into_owned()),
                                    role: "user".to_string(),
                                    name: None,
                                }
                            )
                        } else {
                            let mut parts = Vec::new();
                            parts.push(generated::openai::ChatCompletionRequestUserMessageContentPart::Text(
                                generated::openai::ChatCompletionRequestMessageContentPartText {
                                    text: m.content.into_owned(),
                                    type_: "text".to_string(),
                                }
                            ));
                            if let Some(images) = m.images {
                                for url in images {
                                    parts.push(generated::openai::ChatCompletionRequestUserMessageContentPart::Image(
                                        generated::openai::ChatCompletionRequestMessageContentPartImage {
                                            image_url: generated::openai::ChatCompletionRequestMessageContentPartImageImageUrl {
                                                url: url.into_owned(),
                                                detail: None,
                                            },
                                            type_: "image_url".to_string(),
                                        }
                                    ));
                                }
                            }
                            generated::openai::ChatCompletionRequestMessage::UserMessage(
                                generated::openai::ChatCompletionRequestUserMessage {
                                    content: generated::openai::ChatCompletionRequestUserMessageContent::Array(parts),
                                    role: "user".to_string(),
                                    name: None,
                                }
                            )
                        }
                    }
                }
            }).collect(),
            temperature: req.temperature.map(|v| v as f64),
            max_tokens: req.max_tokens.map(|v| v as i64),
            stream: req.stream,
            // Mandatory collections (not Options)
            functions: Vec::new(),
            logit_bias: std::collections::HashMap::new(),
            tools: {
                let mut v = Vec::new();
                if let Some(tools) = req.tools {
                    for t in tools {
                        let tool = generated::openai::ChatCompletionTool {
                            type_: t.type_.into_owned(),
                            function: generated::openai::FunctionObject {
                                name: t.function.name.into_owned(),
                                description: t.function.description.map(|d| d.into_owned()),
                                parameters: t.function.parameters.and_then(|p| {
                                    serde_json::from_value(p).ok()
                                }),
                                strict: t.function.strict,
                            }
                        };
                        v.push(generated::openai::CreateChatCompletionRequestToolsItem::ChatCompletionTool(tool));
                    }
                }
                v
            },
            // Optional fields (None)
            audio: None,
            frequency_penalty: None,
            function_call: None,
            logprobs: None,
            max_completion_tokens: None,
            modalities: None,
            n: None,
            parallel_tool_calls: None,
            prediction: None,
            presence_penalty: None,
            reasoning_effort: None,
            response_format: req.response_format.and_then(|v| serde_json::from_value(v).ok()),
            seed: None,
            service_tier: None,
            stop: None,
            stream_options: None,
            tool_choice: None,
            top_logprobs: None,
            top_p: None,
            user: None,
            metadata: None,
            prompt_cache_key: None,
            prompt_cache_retention: None,
            store: None,
            safety_identifier: None,
            verbosity: None,
            web_search_options: None,
        }
    }
}

// // Implementation for Anthropic
// impl<'a> From<LiteLLMRequest<'a>> for generated::anthropic::CreateMessageParams {
//     fn from(req: LiteLLMRequest<'a>) -> Self {
//         Self {
//             model: generated::anthropic::Model::from(req.model.as_ref()),
//             messages: req.messages.into_iter().map(|m| {
//                 generated::anthropic::InputMessage {
//                     content: generated::anthropic::InputMessageContent::String(m.content.into_owned()),
//                     role: generated::anthropic::InputMessageRole::User,
//                 }
//             }).collect(),
//             max_tokens: req.max_tokens.unwrap_or(1024) as i64,
//             ..Default::default()
//         }
//     }
// }
