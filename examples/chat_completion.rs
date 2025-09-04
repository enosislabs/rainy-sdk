use rainy_sdk::{AuthConfig, RainyClient, ChatCompletionRequest, ChatMessage, ChatRole};
use std::error::Error;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize client - base URL defaults to api.enosislabs.com
    let client = RainyClient::new(
        AuthConfig::new().with_api_key("your-api-key-here")
    )?;

    println!("💬 Rainy API Chat Example");
    println!("=========================");
    println!("Type 'quit' to exit\n");

    let mut conversation_history: Vec<ChatMessage> = Vec::new();

    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        if input.eq_ignore_ascii_case("quit") {
            break;
        }

        // Add user message to history
        conversation_history.push(ChatMessage {
            role: ChatRole::User,
            content: input.to_string(),
        });

        // Create chat completion request
        let request = ChatCompletionRequest {
            model: "gemini-pro".to_string(),
            messages: conversation_history.clone(),
            max_tokens: Some(500),
            temperature: Some(0.7),
            stream: None,
        };

        match client.create_chat_completion(request).await {
            Ok(response) => {
                if let Some(choice) = response.choices.first() {
                    println!("🤖 Assistant: {}", choice.message.content);

                    // Add assistant response to history
                    conversation_history.push(ChatMessage {
                        role: ChatRole::Assistant,
                        content: choice.message.content.clone(),
                    });

                    println!("📊 Tokens used: {}", response.usage.total_tokens);
                }
            }
            Err(e) => {
                eprintln!("❌ Error: {}", e);

                // Remove the failed user message from history
                conversation_history.pop();
            }
        }

        println!();
    }

    println!("👋 Goodbye!");
    Ok(())
}
