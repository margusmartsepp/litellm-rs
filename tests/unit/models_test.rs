use litellm_rs::models::{LiteLLMRequest, UnifiedMessage};
use std::borrow::Cow;

#[test]
fn test_zero_copy_mapping() {
    let req = LiteLLMRequest {
        model: Cow::Borrowed("claude-4"),
        messages: vec![UnifiedMessage {
            role: Cow::Borrowed("user"),
            content: Cow::Borrowed("Audit this logic."),
            ..Default::default()
        }],
        temperature: Some(0.7),
        ..Default::default()
    };

    let openai_req: litellm_rs::models::generated::openai::CreateChatCompletionRequest = req.into();
    assert_eq!(openai_req.model.to_string(), "claude-4");
}

#[test]
fn test_structured_output_mapping() {
    let mut req = LiteLLMRequest::default();
    req.response_format = Some(serde_json::json!({ "type": "json_object" }));

    let openai_req: litellm_rs::models::generated::openai::CreateChatCompletionRequest = req.into();
    assert!(openai_req.response_format.is_some());
}

#[test]
fn test_tool_mapping() {
    use litellm_rs::models::{UnifiedTool, UnifiedFunction};
    let mut req = LiteLLMRequest::default();
    req.tools = Some(vec![UnifiedTool {
        type_: Cow::Borrowed("function"),
        function: UnifiedFunction {
            name: Cow::Borrowed("get_weather"),
            ..Default::default()
        },
        ..Default::default()
    }]);

    let openai_req: litellm_rs::models::generated::openai::CreateChatCompletionRequest = req.into();
    let tools = &openai_req.tools;
    assert_eq!(tools.len(), 1);
}

#[test]
fn test_vision_content_mapping() {
    let req = LiteLLMRequest {
        messages: vec![UnifiedMessage {
            role: Cow::Borrowed("user"),
            content: Cow::Borrowed("What is this?"),
            images: Some(vec![Cow::Borrowed("data:image/png;base64,ABC")]),
            ..Default::default()
        }],
        ..Default::default()
    };

    let openai_req: litellm_rs::models::generated::openai::CreateChatCompletionRequest = req.into();
    let message = &openai_req.messages[0];

    // For vision, OpenAI uses an array of content parts
    match message {
        litellm_rs::models::generated::openai::ChatCompletionRequestMessage::UserMessage(u) => {
            match &u.content {
                litellm_rs::models::generated::openai::ChatCompletionRequestUserMessageContent::Array(parts) => {
                    assert_eq!(parts.len(), 2); // Text + Image
                },
                _ => panic!("Expected array content for vision"),
            }
        },
        _ => panic!("Expected UserMessage variant"),
    }
}
