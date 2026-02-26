use rainy_sdk::{RainySessionClient, SessionConfig};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let email = std::env::var("RAINY_EMAIL").unwrap_or_else(|_| "user@example.com".to_string());
    let password = std::env::var("RAINY_PASSWORD").unwrap_or_else(|_| "password123".to_string());

    let mut session = RainySessionClient::with_config(SessionConfig::new().with_timeout(30))?;

    println!("🔐 Rainy API v3 Session Example");
    println!("===============================");

    match session.login(&email, &password).await {
        Ok(login) => {
            println!("✅ Logged in as {}", login.user.email);
            println!("🪪 Role: {}", login.user.role);

            match session.org_me().await {
                Ok(org) => println!("🏢 Org: {} (plan={}, region={})", org.name, org.plan_id, org.region),
                Err(e) => println!("⚠️ Could not fetch org profile: {e}"),
            }

            match session.usage_credits().await {
                Ok(credits) => println!("💳 Credits: {} {}", credits.balance, credits.currency),
                Err(e) => println!("⚠️ Could not fetch credits: {e}"),
            }

            match session.list_api_keys().await {
                Ok(keys) => println!("🔑 API keys: {}", keys.len()),
                Err(e) => println!("⚠️ Could not list API keys: {e}"),
            }
        }
        Err(e) => {
            println!("❌ Login failed: {e}");
            println!("Set RAINY_EMAIL and RAINY_PASSWORD to run this example against Rainy API v3.");
        }
    }

    Ok(())
}
