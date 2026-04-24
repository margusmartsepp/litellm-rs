use crate::models::{LiteLLMRequest, generated};
use serde_json;

pub struct Translator;

impl Translator {
    /// Transforms the internal request into a provider-specific JSON string.
    /// This is where we leverage the Generated types from build.rs.
    pub fn to_provider_payload(req: LiteLLMRequest) -> anyhow::Result<(String, String)> {
        let model_prefix = req.model.to_lowercase();

        if model_prefix.contains("gpt") || model_prefix.contains("openai") || model_prefix.contains("gemma") || model_prefix.contains("llama") {
            let openai_req: generated::openai::CreateChatCompletionRequest = req.into();
            let payload = serde_json::to_string(&openai_req)?;
            Ok((payload, "/v1/chat/completions".to_string()))
        } else {
            Err(anyhow::anyhow!("Unsupported provider or Anthropic currently disabled: {}", req.model))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    use crate::models::UnifiedMessage;

    #[test]
    fn test_translation_routing() {
        let req = LiteLLMRequest {
            model: Cow::Borrowed("gpt-4o"),
            messages: vec![UnifiedMessage {
                role: Cow::Borrowed("user"),
                content: Cow::Borrowed("test"),
                ..Default::default()
            }],
            ..Default::default()
        };
        let (json, endpoint) = Translator::to_provider_payload(req).unwrap();
        assert!(json.contains("gpt-4o"));
        assert_eq!(endpoint, "/v1/chat/completions");
    }
}
