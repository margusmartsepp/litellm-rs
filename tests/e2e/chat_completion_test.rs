use std::borrow::Cow;

#[path = "../common/mod.rs"]
mod common;
use common::*;

#[tokio::test]
async fn test_lmstudio_gemma_e2e() -> anyhow::Result<()> {
    // 1. Initialize Client pointing to local LM Studio
    let client = LiteLLM::new("http://localhost:1234", "lm-studio-local");

    // 2. Build Request
    let req = gemma_request("Say 'Hello from Gemma!'");

    // 3. Execute
    println!("Sending request to LM Studio...");
    let response = client.chat(req).await?;

    // 4. Verify
    println!("Received response from model: {}", response.metadata.model_version);
    println!("Content: {}", response.content);

    assert!(!response.content.is_empty());
    assert_eq!(response.metadata.provider, "openai");

    Ok(())
}

#[tokio::test]
async fn test_lmstudio_list_models() -> anyhow::Result<()> {
    let client = LiteLLM::new("http://localhost:1234", "lm-studio-local");

    let models = client.list_models().await?;
    println!("Available Models: {:?}", models);

    assert!(!models.is_empty());
    assert!(models.contains(&"google/gemma-4-e4b".to_string()));

    Ok(())
}

#[tokio::test]
async fn test_lmstudio_streaming() -> anyhow::Result<()> {
    use futures_util::StreamExt;
    let client = LiteLLM::new("http://localhost:1234", "lm-studio-local");

    let req = gemma_request("Count from 1 to 5.");

    println!("Starting stream...");
    let stream = client.stream_chat(req).await?;
    tokio::pin!(stream);
    let mut full_text = String::new();

    while let Some(chunk) = stream.next().await {
        let text = chunk?;
        print!("{}", text);
        full_text.push_str(&text);
    }
    println!("\nStream complete.");

    assert!(!full_text.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_lmstudio_vision() -> anyhow::Result<()> {
    use base64::{Engine as _, engine::general_purpose};
    use std::fs;

    let client = LiteLLM::new("http://localhost:1234", "lm-studio-local");

    // Read the logo file
    let image_path = "img/logo.jpeg";
    let image_data = fs::read(image_path).map_err(|e| anyhow::anyhow!("Failed to read image at {}: {}", image_path, e))?;
    let base64_encoded = general_purpose::STANDARD.encode(image_data);
    let data_url = format!("data:image/jpeg;base64,{}", base64_encoded);

    let req = LiteLLMRequest {
        model: Cow::Borrowed("google/gemma-4-e4b"),
        messages: vec![UnifiedMessage {
            role: Cow::Borrowed("user"),
            content: Cow::Borrowed("Describe this logo in detail. What text do you see and what are the main elements?"),
            images: Some(vec![Cow::Borrowed(&data_url)]),
            ..Default::default()
        }],
        temperature: Some(0.0),
        ..Default::default()
    };

    println!("Sending vision request with logo ({} bytes) to LM Studio...", base64_encoded.len());
    let response = client.chat(req).await?;

    println!("Model Response: {}", response.content);
    assert!(!response.content.is_empty());

    Ok(())
}

#[tokio::test]
async fn test_lmstudio_model_lifecycle() -> anyhow::Result<()> {
    let client = LiteLLM::new("http://localhost:1234", "lm-studio-local");
    let model_id = "google/gemma-4-e4b";

    // 1. Unload
    println!("Unloading model: {}", model_id);
    let _ = client.unload_model(model_id).await;

    // 2. Load
    println!("Loading model: {}", model_id);
    client.load_model(model_id).await?;

    // 3. Verify it's in the list
    let models = client.list_models().await?;
    assert!(models.contains(&model_id.to_string()));

    Ok(())
}

#[tokio::test]
async fn test_lmstudio_tools() -> anyhow::Result<()> {
    let client = LiteLLM::new("http://localhost:1234", "lm-studio-local");

    let mut req = gemma_request("What's the weather in Tallinn?");
    req.tools = Some(vec![UnifiedTool {
        type_: Cow::Borrowed("function"),
        function: UnifiedFunction {
            name: Cow::Borrowed("get_weather"),
            description: Some(Cow::Borrowed("Get the current weather in a given location")),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "location": {
                        "type": "string",
                        "description": "The city and state, e.g. San Francisco, CA"
                    }
                },
                "required": ["location"]
            })),
            ..Default::default()
        },
        ..Default::default()
    }]);

    println!("Sending tool-use request to LM Studio...");
    let response = client.chat(req).await?;

    println!("Content: {}", response.content);
    if let Some(tool_calls) = response.tool_calls {
        for call in tool_calls {
            println!("Tool Call: {} with args: {}", call.name, call.arguments);
            assert_eq!(call.name, "get_weather");
        }
    } else {
        println!("No tool calls returned, model might have described it instead.");
    }

    Ok(())
}
