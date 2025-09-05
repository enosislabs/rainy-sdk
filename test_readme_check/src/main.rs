use rainy_sdk::{RainyClient, ChatCompletionRequest, ChatMessage, ChatRole};
use std::error::Error;

// This function contains the example from README
#[allow(dead_code)]
async fn readme_example() -> Result<(), Box<dyn Error>> {
    // Initialize client with your API key - automatically connects to api.enosislabs.com
    let client = RainyClient::with_api_key("your-api-key-here")?;

    // Create a simple chat completion
    let messages = vec![
        ChatMessage {
            role: ChatRole::User,
            content: "Hello! Tell me a joke.".to_string(),
        }
    ];

    let request = ChatCompletionRequest {
        model: "gemini-pro".to_string(),
        messages,
        max_tokens: Some(150),
        temperature: Some(0.7),
        stream: None,
    };

    let _response = client.create_chat_completion(request).await?;
    // println!("Response: {}", response.choices[0].message.content);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    println!("README example code compiles successfully!");
    readme_example().await?;
    Ok(())
}
