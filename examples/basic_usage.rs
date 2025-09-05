use rainy_sdk::{AuthConfig, ChatCompletionRequest, ChatMessage, RainyClient};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the client - base URL automatically set to api.enosislabs.com
    let client = RainyClient::with_config(
        AuthConfig::new("your-api-key-here")
            .with_timeout(30)
    )?;

    println!("🌟 Rainy API SDK Example");
    println!("========================");

    // Check API health
    println!("\n1. Checking API health...");
    match client.health_check().await {
        Ok(health) => {
            println!("✅ API Status: {:?}", health.status);
            println!("⏱️  Uptime: {:.2}s", health.uptime);
            println!(
                "🔗 Services: Database={}, Redis={:?}, Providers={}",
                health.services.database, health.services.redis, health.services.providers
            );
        }
        Err(e) => {
            println!("❌ Health check failed: {e}");
            return Ok(());
        }
    }

    // Get user account information
    println!("\n2. Getting user account info...");
    match client.get_user_account().await {
        Ok(user) => {
            println!("👤 User ID: {}", user.user_id);
            println!("📊 Plan: {}", user.plan_name);
            println!("💰 Current Credits: {:.2}", user.current_credits);
            println!("📈 Used This Month: {:.2}", user.credits_used_this_month);
        }
        Err(e) => {
            println!("❌ Failed to get user account: {e}");
        }
    }

    // Create a chat completion
    println!("\n3. Creating chat completion...");
    let messages = vec![ChatMessage::user("Hello! Can you tell me a short joke?")];

    let request = ChatCompletionRequest::new("gemini-pro", messages)
        .with_max_tokens(150)
        .with_temperature(0.7);

    match client.create_chat_completion(request).await {
        Ok(response) => {
            println!("🤖 Response:");
            if let Some(choice) = response.choices.first() {
                println!("   {}", choice.message.content);
            }
            if let Some(usage) = &response.usage {
                println!("📊 Usage: {} tokens", usage.total_tokens);
            }
        }
        Err(e) => {
            println!("❌ Chat completion failed: {e}");
        }
    }

    // Get usage statistics
    println!("\n4. Getting usage statistics...");
    match client.get_usage_stats(Some(7)).await {
        Ok(usage) => {
            println!("📈 Total Requests: {}", usage.total_requests);
            println!("🎫 Total Tokens: {}", usage.total_tokens);
            println!("📅 Daily Usage (last {} days):", usage.daily_usage.len());

            for daily in usage.daily_usage.iter().take(3) {
                println!(
                    "   {}: {:.2} credits, {} requests",
                    daily.date, daily.credits_used, daily.requests
                );
            }
        }
        Err(e) => {
            println!("❌ Failed to get usage stats: {e}");
        }
    }

    // List API keys
    println!("\n5. Listing API keys...");
    match client.list_api_keys().await {
        Ok(keys) => {
            println!("🔑 Found {} API keys:", keys.len());
            for key in keys.iter().take(3) {
                println!(
                    "   - {}: {} (Active: {})",
                    key.key.chars().take(20).collect::<String>() + "...",
                    key.description.as_deref().unwrap_or("No description"),
                    key.is_active
                );
            }
        }
        Err(e) => {
            println!("❌ Failed to list API keys: {e}");
        }
    }

    println!("\n🎉 Example completed!");
    Ok(())
}
