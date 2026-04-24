use litellm_rs::prelude::*;
use std::borrow::Cow;
use serde_json::json;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Verified with OpenAI GPT-4o
    // Read API key from environment variable (Best practice)
    // To set in PowerShell: $env:OPENAI_API_KEY = "your-key-here"
    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("OPENAI_API_KEY not set. Run: $env:OPENAI_API_KEY = 'your-key'");

    let client = LiteLLM::new("https://api.openai.com/v1", &api_key);

    let req = LiteLLMRequest {
        model: Cow::Borrowed("gpt-4o"),
        messages: vec![UnifiedMessage {
            role: Cow::Borrowed("user"),
            content: Cow::Borrowed("Generate a structured profile for a fictional character named 'Aria'."),
            ..Default::default()
        }],
        // Force structured JSON output via response_format
        response_format: Some(json!({
            "type": "json_schema",
            "json_schema": {
                "name": "character_profile",
                "strict": true,
                "schema": {
                    "type": "object",
                    "properties": {
                        "name": { "type": "string" },
                        "class": { "type": "string", "enum": ["Warrior", "Mage", "Rogue"] },
                        "level": { "type": "integer" },
                        "abilities": { "type": "array", "items": { "type": "string" } }
                    },
                    "required": ["name", "class", "level", "abilities"],
                    "additionalProperties": false
                }
            }
        })),
        ..Default::default()
    };

    println!("Sending request for structured output...");
    let response = client.chat(req).await?;

    println!("Raw JSON Response: {}", response.content);

    // Lazy developers can now just parse response.content into a struct!

    Ok(())
}
