#![allow(dead_code)]
#![allow(unused_imports)]
pub use litellm_rs::prelude::*;
pub use litellm_rs::models::{UnifiedTool, UnifiedFunction};
pub use wiremock::matchers::{method, path, header};
pub use wiremock::{Mock, MockServer, ResponseTemplate};
pub use std::borrow::Cow;
pub use serde_json::json;

pub fn create_test_request<'a>(model: &'a str, content: &'a str) -> LiteLLMRequest<'a> {
    LiteLLMRequest {
        model: Cow::Borrowed(model),
        messages: vec![UnifiedMessage {
            role: Cow::Borrowed("user"),
            content: Cow::Borrowed(content),
            images: None,
        }],
        ..Default::default()
    }
}

pub fn gemma_request<'a>(content: &'a str) -> LiteLLMRequest<'a> {
    let mut req = create_test_request("google/gemma-4-e4b", content);
    req.temperature = Some(0.0);
    req
}

pub fn openai_request<'a>(content: &'a str) -> LiteLLMRequest<'a> {
    create_test_request("gpt-4o", content)
}

pub fn cerebras_request<'a>(content: &'a str) -> LiteLLMRequest<'a> {
    create_test_request("llama3.1-8b", content)
}

pub fn mock_openai_response(content: &str) -> serde_json::Value {
    json!({
        "id": "chat-123",
        "object": "chat.completion",
        "created": 1677652288,
        "model": "gpt-4o",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": content,
                "refusal": ""
            },
            "logprobs": null,
            "finish_reason": "stop"
        }],
        "usage": {
            "prompt_tokens": 9,
            "completion_tokens": 12,
            "total_tokens": 21
        }
    })
}

pub fn mock_error_response(message: &str) -> serde_json::Value {
    json!({
        "error": {
            "message": message,
            "type": "invalid_request_error",
            "param": null,
            "code": null
        }
    })
}
