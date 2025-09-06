//! Error handling example
//!
//! This example demonstrates:
//! - Different types of errors
//! - Retry logic
//! - Error recovery strategies

use rainy_sdk::{
    models, retry_with_backoff, ChatCompletionRequest, ChatMessage, RainyClient, RainyError,
    RetryConfig,
};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = std::env::var("RAINY_API_KEY").unwrap_or_else(|_| "ra-test-key".to_string());

    let client = RainyClient::with_api_key(&api_key)?;

    println!("🌧️  Rainy SDK v0.2.0 - Error Handling Example");
    println!("==============================================");

    // Example 1: Handle different error types
    println!("\n🚨 Example 1: Error type handling");
    match client.simple_chat("invalid-model-name", "Hello").await {
        Ok(_) => println!("✅ Unexpected success"),
        Err(error) => match error {
            RainyError::InvalidRequest { code, message, .. } => {
                println!("❌ Invalid request [{}]: {}", code, message);
            }
            RainyError::Authentication { message, .. } => {
                println!("❌ Authentication error: {}", message);
            }
            RainyError::Provider {
                provider,
                message,
                retryable,
                ..
            } => {
                println!(
                    "❌ Provider error [{}]: {} (retryable: {})",
                    provider, message, retryable
                );
            }
            _ => {
                println!("❌ Other error: {}", error);
                println!("   Retryable: {}", error.is_retryable());
                if let Some(code) = error.code() {
                    println!("   Code: {}", code);
                }
            }
        },
    }

    // Example 2: Manual retry with backoff
    println!("\n🔄 Example 2: Manual retry logic");
    let retry_config = RetryConfig::new(3);

    let result = retry_with_backoff(&retry_config, || async {
        // Simulate a potentially failing operation
        client
            .simple_chat(models::model_constants::GPT_4O, "Tell me a joke")
            .await
    })
    .await;

    match result {
        Ok(response) => println!(
            "✅ Success after retries: {}",
            response.chars().take(50).collect::<String>()
        ),
        Err(e) => println!("❌ Failed after all retries: {}", e),
    }

    // Example 3: Rate limit handling
    println!("\n⏱️  Example 3: Rate limit handling");
    for i in 1..=5 {
        match client
            .simple_chat(
                models::model_constants::GPT_4O,
                &format!("Quick question #{}", i),
            )
            .await
        {
            Ok(response) => println!(
                "✅ Request {}: {}",
                i,
                response.chars().take(50).collect::<String>()
            ),
            Err(RainyError::RateLimit {
                retry_after,
                message,
                ..
            }) => {
                println!("⏳ Rate limited: {}", message);
                if let Some(wait_time) = retry_after {
                    println!("   Waiting {} seconds...", wait_time);
                    tokio::time::sleep(Duration::from_secs(wait_time)).await;
                }
            }
            Err(e) => println!("❌ Request {} failed: {}", i, e),
        }
    }

    // Example 4: Structured error details
    println!("\n📋 Example 4: Structured error details");
    let request = ChatCompletionRequest::new(
        models::model_constants::GPT_4O,
        vec![ChatMessage::user("Test message")],
    );
    match client.chat_completion(request).await {
        Ok((response, metadata)) => {
            println!("✅ Success with metadata:");
            println!("   Provider: {:?}", metadata.provider);
            println!(
                "   Response time: {}ms",
                metadata.response_time.unwrap_or_default()
            );
            if let Some(tokens) = metadata.tokens_used {
                println!("   Tokens used: {}", tokens);
            }
        }
        Err(RainyError::InsufficientCredits {
            current_credits,
            estimated_cost,
            reset_date,
            ..
        }) => {
            println!("💰 Insufficient credits:");
            println!("   Current: {:.4}", current_credits);
            println!("   Required: {:.4}", estimated_cost);
            if let Some(reset) = reset_date {
                println!("   Resets: {}", reset);
            }
        }
        Err(e) => println!("❌ Other error: {}", e),
    }

    Ok(())
}
