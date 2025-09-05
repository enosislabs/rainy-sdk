use rainy_sdk::{AuthConfig, ChatCompletionRequest, ChatMessage, RainyClient};
use std::error::Error;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize client - base URL defaults to api.enosislabs.com
    let client = RainyClient::new(AuthConfig::new().with_api_key("your-api-key-here"))?;

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
        conversation_history.push(ChatMessage::user(input));

        // Create chat completion request
        let request = ChatCompletionRequest::new("gemini-pro", conversation_history.clone())
            .with_max_tokens(500)
            .with_temperature(0.7);

        match client.create_chat_completion(request).await {
            Ok(response) => {
                if let Some(choice) = response.choices.first() {
                    println!("🤖 Assistant: {}", choice.message.content);

                    // Add assistant response to history
                    conversation_history.push(ChatMessage::assistant(choice.message.content.clone()));

                    if let Some(usage) = &response.usage {
                        println!("📊 Tokens used: {}", usage.total_tokens);
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Error: {e}");

                // Remove the failed user message from history
                conversation_history.pop();
            }
        }

        println!();
    }

    println!("👋 Goodbye!");
    Ok(())
}
