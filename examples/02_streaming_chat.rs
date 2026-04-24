use litellm_rs::prelude::*;
use futures_util::StreamExt;
use std::borrow::Cow;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Read API key from environment variable (Best practice)
    // To set in PowerShell: $env:CEREBRAS_API_KEY = "your-key-here"
    let api_key = std::env::var("CEREBRAS_API_KEY")
        .expect("CEREBRAS_API_KEY not set. Run: $env:CEREBRAS_API_KEY = 'your-key'");

    let client = LiteLLM::new("https://api.cerebras.ai", &api_key);

    let req = LiteLLMRequest {
        model: Cow::Borrowed("llama3.1-8b"),
        messages: vec![UnifiedMessage {
            role: Cow::Borrowed("user"),
            content: Cow::Borrowed("Write a short poem about Rust memory safety."),
            ..Default::default()
        }],
        stream: Some(true),
        ..Default::default()
    };

    println!("Starting stream...");
    let stream = client.stream_chat(req).await?;
    tokio::pin!(stream);

    while let Some(chunk) = stream.next().await {
        print!("{}", chunk?);
    }
    println!("\nStream finished.");

    Ok(())
}
