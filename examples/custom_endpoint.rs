//! Use the protocol client with a custom versioned compatible endpoint.

use rainy_sdk::{AuthConfig, ChatCompletionRequest, ChatMessage, RainyClient};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let config = AuthConfig::new(
        std::env::var("COMPATIBLE_API_KEY").unwrap_or_else(|_| "your-api-key-here".to_string()),
    )
    .with_api_base_url("https://example-compatible-provider.com/v1");
    let client = RainyClient::with_config(config)?;
    let response = client
        .create_chat_completion(ChatCompletionRequest::new(
            "compatible/chat-model",
            vec![ChatMessage::user("Hello")],
        ))
        .await?;
    println!("{}", response.choices[0].message.content);
    Ok(())
}
