use litellm_rs::prelude::*;
use std::borrow::Cow;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Works with Cerebras, OpenAI, Groq, or LM Studio
    // Read API key from environment variable (Best practice)
    // To set in PowerShell: $env:CEREBRAS_API_KEY = "your-key-here"
    let api_key = std::env::var("CEREBRAS_API_KEY")
        .expect("CEREBRAS_API_KEY not set. Run: $env:CEREBRAS_API_KEY = 'your-key'");

    let client = LiteLLM::new("https://api.cerebras.ai", &api_key);

    let req = LiteLLMRequest {
        model: Cow::Borrowed("llama3.1-8b"),
        messages: vec![UnifiedMessage {
            role: Cow::Borrowed("user"),
            content: Cow::Borrowed("What is the speed of light?"),
            ..Default::default()
        }],
        ..Default::default()
    };

    println!("Sending request...");
    let response = client.chat(req).await?;

    println!("Response: {}", response.content);
    println!("Provider: {}", response.metadata.provider);

    Ok(())
}
