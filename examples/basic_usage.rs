use rainy_sdk::{AuthConfig, ChatCompletionRequest, ChatMessage, RainyClient};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize the API-key client for Rainy API v3 (models/chat/search endpoints)
    let client = RainyClient::with_config(AuthConfig::new("your-api-key-here").with_timeout(30))?;

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

    // List models (v3 canonical models endpoint)
    println!("\n2. Listing available models...");
    match client.get_available_models().await {
        Ok(models) => {
            println!("🧠 Total Models: {}", models.total_models);
            println!("🏷️  Active Providers: {:?}", models.active_providers);
        }
        Err(e) => {
            println!("❌ Failed to list models: {e}");
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

    println!("\n4. Tip: JWT/session endpoints (auth, keys, usage, orgs) now live on RainySessionClient.");
    println!("   See examples/session_auth.rs for v3 session usage.");

    println!("\n🎉 Example completed!");
    Ok(())
}
