//! Interactive streaming Chat Completions example.

use futures::StreamExt;
use rainy_sdk::{ChatCompletionRequest, ChatMessage, RainyClient};
use std::error::Error;
use std::io::{self, Write};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = RainyClient::with_api_key(
        std::env::var("RAINY_API_KEY").unwrap_or_else(|_| "your-api-key-here".to_string()),
    )?;

    println!("Rainy SDK streaming Chat Completions example");
    println!("Type 'quit' to exit.\n");

    let mut conversation_history = Vec::new();
    loop {
        print!("You: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();
        if input.eq_ignore_ascii_case("quit") {
            break;
        }
        if input.is_empty() {
            continue;
        }

        conversation_history.push(ChatMessage::user(input));
        let request =
            ChatCompletionRequest::new("compatible/chat-model", conversation_history.clone());

        print!("Assistant: ");
        io::stdout().flush()?;
        let mut stream = match client.chat_completion_stream(request).await {
            Ok(stream) => stream,
            Err(error) => {
                eprintln!("request failed: {error}");
                conversation_history.pop();
                continue;
            }
        };

        let mut assistant = String::new();
        while let Some(chunk) = stream.next().await {
            let chunk = chunk?;
            if let Some(content) = chunk
                .choices
                .first()
                .and_then(|choice| choice.delta.content.as_deref())
            {
                print!("{content}");
                io::stdout().flush()?;
                assistant.push_str(content);
            }
        }
        println!();
        conversation_history.push(ChatMessage::assistant(assistant));
    }

    Ok(())
}
