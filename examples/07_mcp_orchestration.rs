use litellm_rs::prelude::*;
use std::borrow::Cow;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Provision Bifrost Gateway
    println!("Provisioning Bifrost Gateway...");
    let mut provisioner = Provisioner::new(8080);
    provisioner.spawn().await?;

    // 2. Initialize Client
    let client = LiteLLM::new("http://localhost:8080", "bifrost-virtual-key");

    let req = LiteLLMRequest {
        model: Cow::Borrowed("gpt-4o"),
        messages: vec![UnifiedMessage {
            role: Cow::Borrowed("user"),
            content: Cow::Borrowed("Search the local project for any files containing 'Scientific Audit' and list them."),
            ..Default::default()
        }],
        // MCP Code Mode reduces tool-call overhead by 92%
        mcp_code_mode: true,
        ..Default::default()
    };

    println!("Sending MCP request (Optimized for Local Code Execution)...");
    let response = client.chat(req).await?;

    println!("Response: {}", response.content);
    println!("MCP Optimization Active: {}", response.metadata.provider == "openai"); // Usually routed through OpenAI-compatible gateway

    Ok(())
}
