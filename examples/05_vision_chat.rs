use litellm_rs::prelude::*;
use std::borrow::Cow;
use base64::{Engine as _, engine::general_purpose};
use std::fs;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let client = LiteLLM::new("http://localhost:1234", "local-key");

    // 1. Load and encode image
    let img_path = "img/logo.jpeg";
    let img_data = fs::read(img_path).map_err(|e| anyhow::anyhow!("Failed to read image at {}: {}", img_path, e))?;
    let base64_img = general_purpose::STANDARD.encode(img_data);
    let data_url = format!("data:image/jpeg;base64,{}", base64_img);

    // 2. Build Request with Multi-Modal Image Input
    let req = LiteLLMRequest {
        model: Cow::Borrowed("google/gemma-4-e4b"),
        messages: vec![UnifiedMessage {
            role: Cow::Borrowed("user"),
            content: Cow::Borrowed("Describe the colors and main elements in this logo image."),
            images: Some(vec![Cow::Borrowed(&data_url)]),
            ..Default::default()
        }],
        ..Default::default()
    };

    println!("Sending vision request with image ({} bytes)...", base64_img.len());
    let response = client.chat(req).await?;

    println!("Response: {}", response.content);

    Ok(())
}
