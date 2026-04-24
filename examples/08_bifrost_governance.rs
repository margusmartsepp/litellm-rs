use litellm_rs::prelude::*;
use std::borrow::Cow;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Provision Bifrost Gateway automatically via npx
    println!("Provisioning Bifrost Gateway on port 8080...");
    let mut provisioner = Provisioner::new(8080);
    provisioner.spawn().await?;

    // 2. Initialize Client pointing to the provisioned gateway
    let client = LiteLLM::new("http://localhost:8080", "agent-production-key");

    // 2. Build Request with Governance Headers
    let mut extra_headers = std::collections::HashMap::new();
    extra_headers.insert("x-bf-budget-limit".to_string(), "0.05".to_string()); // Limit this specific request to $0.05
    extra_headers.insert("x-bf-metadata".to_string(), "{\"agent_id\": \"lazy-dev-01\"}".to_string());

    let req = LiteLLMRequest {
        model: Cow::Borrowed("gpt-4o"),
        messages: vec![UnifiedMessage {
            role: Cow::Borrowed("user"),
            content: Cow::Borrowed("This is a request that might trigger a cache hit or failover."),
            ..Default::default()
        }],
        extra_headers: Some(extra_headers),
        ..Default::default()
    };

    println!("Sending request through Bifrost Gateway...");
    let response = client.chat(req).await?;

    let meta = &response.metadata;
    println!("--- Governance & Resilience Status ---");
    println!("Cache Hit:        {}", meta.cache_hit);
    println!("Failover Status:  {}", meta.failover);
    println!("Health State:     {:?}", meta.health_state);
    println!("Budget Status:    {}", if meta.budget_exceeded { "EXCEEDED" } else { "OK" });

    Ok(())
}
