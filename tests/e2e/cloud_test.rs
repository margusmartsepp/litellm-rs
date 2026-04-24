use std::time::Instant;

#[path = "../common/mod.rs"]
mod common;
use common::*;

#[tokio::test]
#[ignore] // Run manually with CEREBRAS_API_KEY env var
async fn test_cerebras_cloud_e2e() -> anyhow::Result<()> {
    // 1. Initialize Client pointing to Cerebras Cloud
    let api_key = std::env::var("CEREBRAS_API_KEY")
        .map_err(|_| anyhow::anyhow!("CEREBRAS_API_KEY not set"))?;

    let client = LiteLLM::new("https://api.cerebras.ai", &api_key);

    // 2. Build Request
    let req = cerebras_request("Explain the significance of Llama 3.1 in 3 bullet points.");

    // 3. Execute and Measure Speed
    println!("Sending request to Cerebras Cloud...");
    let start = Instant::now();
    let response = client.chat(req).await?;
    let duration = start.elapsed();

    // 4. Verify and Log Stats
    println!("Received response from model: {}", response.metadata.model_version);
    let usage = response.metadata.usage.as_ref();
    let prompt_tokens = usage.and_then(|u| u.get("prompt_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);
    let completion_tokens = usage.and_then(|u| u.get("completion_tokens")).and_then(|v| v.as_i64()).unwrap_or(0);

    println!("Content: {}", response.content);
    println!("Usage: {} prompt, {} completion tokens", prompt_tokens, completion_tokens);
    println!("Latency: {:?}", duration);

    let tps = completion_tokens as f64 / duration.as_secs_f64();
    println!("Speed: {:.2} tokens/sec", tps);

    assert!(!response.content.is_empty());
    assert_eq!(response.metadata.provider, "openai");

    Ok(())
}

#[tokio::test]
#[ignore] // Run manually with CEREBRAS_API_KEY env var
async fn test_cerebras_streaming_throughput() -> anyhow::Result<()> {
    use futures_util::StreamExt;

    let api_key = std::env::var("CEREBRAS_API_KEY")
        .map_err(|_| anyhow::anyhow!("CEREBRAS_API_KEY not set"))?;

    let client = LiteLLM::new("https://api.cerebras.ai", &api_key);

    let req = cerebras_request("Write a long story about a robot learning to paint.");

    println!("Starting high-speed stream from Cerebras...");
    let start = Instant::now();
    let stream = client.stream_chat(req).await?;
    tokio::pin!(stream);

    let mut token_count = 0;
    let mut full_text = String::new();

    while let Some(chunk) = stream.next().await {
        let text = chunk?;
        token_count += 1; // Simplification: count chunks as tokens for now
        full_text.push_str(&text);
        if token_count % 50 == 0 {
            print!(".");
            use std::io::{Write, stdout};
            stdout().flush()?;
        }
    }

    let duration = start.elapsed();
    let tps = token_count as f64 / duration.as_secs_f64();

    println!("\nStream complete.");
    println!("Total chunks: {}", token_count);
    println!("Total time: {:?}", duration);
    println!("Estimated Throughput: {:.2} chunks/sec", tps);

    assert!(!full_text.is_empty());
    assert!(token_count > 10);

    Ok(())
}
