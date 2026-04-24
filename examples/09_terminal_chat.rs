use litellm_rs::prelude::*;
use std::io::{self, Write};
use std::borrow::Cow;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Verified with Cerebras Cloud (High speed inference)
    // Read API key from environment variable (Best practice)
    // To set in PowerShell: $env:CEREBRAS_API_KEY = "your-key-here"
    let api_key = std::env::var("CEREBRAS_API_KEY")
        .expect("CEREBRAS_API_KEY not set. Run: $env:CEREBRAS_API_KEY = 'your-key'");

    let client = LiteLLM::new("https://api.cerebras.ai", &api_key);

    // 1. Initialize conversation with a custom system prompt
    let mut history = vec![
        UnifiedMessage {
            role: Cow::Borrowed("system"),
            content: Cow::Borrowed("You are a helpful AI assistant. You must respond in a concise, professional manner. If the user asks about Rust, emphasize its memory safety."),
            ..Default::default()
        }
    ];

    println!("--- Terminal Chat Session (Type 'exit' or 'quit' to end) ---");

    loop {
        // 2. Capture user input
        print!("\n> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.to_lowercase() == "exit" || input.to_lowercase() == "quit" {
            println!("Goodbye!");
            break;
        }

        // 3. Append user message to history
        history.push(UnifiedMessage {
            role: Cow::Borrowed("user"),
            content: Cow::Owned(input.to_string()),
            ..Default::default()
        });

        // 4. Build and send request
        let req = LiteLLMRequest {
            model: Cow::Borrowed("gpt-oss-120b"),
            messages: history.clone(), // Pass the entire conversation context
            temperature: Some(0.7),
            ..Default::default()
        };

        print!("AI: ");
        io::stdout().flush()?;

        let response = client.chat(req).await?;
        println!("{}", response.content);

        // 5. Append assistant response to history for multi-turn context
        history.push(UnifiedMessage {
            role: Cow::Borrowed("assistant"),
            content: Cow::Owned(response.content),
            ..Default::default()
        });
    }

    Ok(())
}
