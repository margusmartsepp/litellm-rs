use litellm_rs::prelude::*;
use std::borrow::Cow;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Verified with Cerebras Cloud (High speed inference)
    // Read API key from environment variable (Best practice)
    // To set in PowerShell: $env:CEREBRAS_API_KEY = "your-key-here"
    let api_key = std::env::var("CEREBRAS_API_KEY")
        .expect("CEREBRAS_API_KEY not set. Run: $env:CEREBRAS_API_KEY = 'your-key'");

    let client = LiteLLM::new("https://api.cerebras.ai", &api_key);

    let req = LiteLLMRequest {
        model: Cow::Borrowed("llama3.1-8b"),
        messages: vec![UnifiedMessage {
            role: Cow::Borrowed("user"),
            content: Cow::Borrowed("Calculate the first 10 prime numbers and explain the process."),
            ..Default::default()
        }],
        ..Default::default()
    };

    println!("Sending request for performance auditing...");
    let response = client.chat(req).await?;
    let meta = &response.metadata;

    println!("--- Scientific Audit Trail ---");
    println!("Provider:    {}", meta.provider);
    println!("Model:       {}", meta.model_version);
    println!("Latency:     {:?}", meta.latency);
    println!("Request ID:  {:?}", meta.id);
    println!("Finish:      {:?}", meta.finish_reason);

    if let Some(usage) = &meta.usage {
        println!("Token Usage: {}", serde_json::to_string_pretty(usage)?);

        // Calculate Throughput (Tokens Per Second)
        let comp_tokens = usage.get("completion_tokens")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let tps = comp_tokens / meta.latency.as_secs_f64();
        println!("Actual TPS:  {:.2} tokens/sec", tps);
    }

    Ok(())
}
