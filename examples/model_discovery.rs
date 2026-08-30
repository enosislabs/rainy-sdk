//! Explicit Rainy model discovery and local public-capability filtering.

use rainy_sdk::{ModelSelectionCriteria, RainyClient};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = RainyClient::with_api_key(
        std::env::var("RAINY_API_KEY").unwrap_or_else(|_| "your-api-key-here".to_string()),
    )?;
    let models = client
        .select_models(ModelSelectionCriteria {
            required_input_modalities: vec!["text".to_string()],
            require_tools: Some(true),
            ..Default::default()
        })
        .await?;
    for model in models {
        println!("{}", model.id);
    }
    Ok(())
}
