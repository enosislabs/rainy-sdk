//! Native Anthropic-compatible Messages example.

use rainy_sdk::{AnthropicMessage, AnthropicMessageRequest, AnthropicTool, RainyClient};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = RainyClient::with_api_key(
        std::env::var("RAINY_API_KEY").unwrap_or_else(|_| "your-api-key-here".to_string()),
    )?;
    let request = AnthropicMessageRequest::new(
        "compatible/messages-model",
        vec![AnthropicMessage::user("Give a concise answer.")],
        1024,
    )
    .with_system("You are a concise assistant.")
    // Native Messages thinking uses an explicit budget. It is not an effort
    // label and is not translated into OpenAI-compatible reasoning.
    .with_thinking(256)
    .with_tools(vec![AnthropicTool::new(
        "lookup",
        rainy_sdk::serde_json::json!({
            "type": "object",
            "properties": {"key": {"type": "string"}},
            "required": ["key"]
        }),
    )]);

    let response = client.create_message(request).await?;
    println!("{}", response.text());
    Ok(())
}
