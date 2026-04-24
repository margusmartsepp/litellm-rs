use std::time::Duration;
use tokio;
use std::borrow::Cow;

#[path = "../common/mod.rs"]
mod common;
use common::*;

#[tokio::test]
async fn test_full_stack_orchestration() -> anyhow::Result<()> {
    let mock_server = MockServer::start().await;

    let mock_response = ResponseTemplate::new(200)
        .append_header("x-litellm-response-cost", "0.00042")
        .append_header("x-litellm-model-id", "gpt-4o-audit-v1")
        .set_body_json(serde_json::json!({
            "id": "chat-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "gpt-4o",
            "choices": [{
                "index": 0,
                "message": {
                    "role": "assistant",
                    "content": "Logic verified. System stable.",
                    "refusal": ""
                },
                "logprobs": {
                    "content": [],
                    "refusal": []
                },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 9,
                "completion_tokens": 12,
                "total_tokens": 21
            }
        }));

    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .and(header("x-bf-vk", "vk-larynx-dev"))
        .respond_with(mock_response)
        .mount(&mock_server)
        .await;

    let client = LiteLLM::new(&mock_server.uri(), "vk-larynx-dev");

    let req = openai_request("Run integration audit.");

    let response = client.chat(req).await?;

    assert_eq!(response.content, "Logic verified. System stable.");
    assert_eq!(response.metadata.cost, 0.00042);
    assert_eq!(response.metadata.model_version, "gpt-4o-audit-v1");
    assert_eq!(response.metadata.provider, "openai");
    assert!(response.metadata.latency < Duration::from_millis(100));

    Ok(())
}

#[tokio::test]
async fn test_unsupported_model_error() {
    let client = LiteLLM::new("http://localhost:8080", "unused");
    let req = LiteLLMRequest {
        model: Cow::Borrowed("unknown-model-99"),
        messages: vec![],
        ..Default::default()
    };

    let result = client.chat(req).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Unsupported provider"));
}

#[tokio::test]
async fn test_bifrost_semantic_caching() -> anyhow::Result<()> {
    let mock_server = MockServer::start().await;

    let cache_hit_response = ResponseTemplate::new(200)
        .append_header("x-litellm-response-cost", "0.0")
        .append_header("x-bf-cache-status", "hit")
        .set_body_json(serde_json::json!({
            "id": "chat-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "gpt-4o",
            "choices": [{"index": 0, "message": {"content": "Cached Answer", "role": "assistant"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 0, "completion_tokens": 0, "total_tokens": 0}
        }));

    Mock::given(method("POST"))
        .respond_with(cache_hit_response)
        .mount(&mock_server)
        .await;

    let client = LiteLLM::new(&mock_server.uri(), "vk-test");
    let req = openai_request("Repeat question");

    let response = client.chat(req).await?;
    assert!(response.metadata.cache_hit);
    assert_eq!(response.metadata.cost, 0.0);
    assert_eq!(response.content, "Cached Answer");

    Ok(())
}

#[tokio::test]
async fn test_bifrost_governance_budget() -> anyhow::Result<()> {
    let mock_server = MockServer::start().await;

    let budget_error = ResponseTemplate::new(429)
        .append_header("x-bf-budget-exceeded", "true")
        .set_body_json(serde_json::json!({ "error": "Budget exceeded" }));

    Mock::given(method("POST"))
        .respond_with(budget_error)
        .mount(&mock_server)
        .await;

    let client = LiteLLM::new(&mock_server.uri(), "vk-capped");
    let req = openai_request("Expensive task");

    let response = client.chat(req).await?;
    assert!(response.metadata.budget_exceeded);

    Ok(())
}

#[tokio::test]
async fn test_bifrost_adaptive_failover() -> anyhow::Result<()> {
    let mock_server = MockServer::start().await;

    let failover_response = ResponseTemplate::new(200)
        .append_header("x-bf-failover", "true")
        .append_header("x-bf-health-state", "Degraded")
        .set_body_json(serde_json::json!({
            "id": "chat-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "gpt-4o",
            "choices": [{"index": 0, "message": {"content": "Failover Answer", "role": "assistant"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 9, "completion_tokens": 12, "total_tokens": 21}
        }));

    Mock::given(method("POST"))
        .respond_with(failover_response)
        .mount(&mock_server)
        .await;

    let client = LiteLLM::new(&mock_server.uri(), "vk-failover");
    let req = openai_request("Reliability test");

    let response = client.chat(req).await?;
    assert!(response.metadata.failover);
    assert_eq!(response.metadata.health_state.as_deref(), Some("Degraded"));

    Ok(())
}

#[tokio::test]
async fn test_bifrost_mcp_code_mode() -> anyhow::Result<()> {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(header("x-bf-mcp-code-mode", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "chat-123",
            "object": "chat.completion",
            "created": 1677652288,
            "model": "gpt-4o",
            "choices": [{"index": 0, "message": {"content": "MCP Answer", "role": "assistant"}, "finish_reason": "stop"}],
            "usage": {"prompt_tokens": 9, "completion_tokens": 12, "total_tokens": 21}
        })))
        .mount(&mock_server)
        .await;

    let client = LiteLLM::new(&mock_server.uri(), "vk-mcp");
    let req = openai_request("Code mode test");
    let mut req = req;
    req.mcp_code_mode = true;

    let response = client.chat(req).await?;
    assert_eq!(response.content, "MCP Answer");

    Ok(())
}
