use litellm_rs::prelude::*;
use std::borrow::Cow;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Requires an OpenAI-compatible provider that supports tool use (e.g., LM Studio, OpenAI, Groq)
    let client = LiteLLM::new("http://localhost:1234", "local-key");

    let mut req = LiteLLMRequest {
        model: Cow::Borrowed("google/gemma-4-e4b"),
        messages: vec![UnifiedMessage {
            role: Cow::Borrowed("user"),
            content: Cow::Borrowed("What's the weather like in Tallinn, Estonia?"),
            ..Default::default()
        }],
        ..Default::default()
    };

    // Define a tool
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
    }]);

    println!("Sending tool-use request...");
    let response = client.chat(req).await?;

    if let Some(tool_calls) = response.tool_calls {
        for call in tool_calls {
            println!("Model called tool: {}", call.name);
            println!("Arguments: {}", call.arguments);
        }
    } else {
        println!("No tool calls returned. Response: {}", response.content);
    }

    Ok(())
}
