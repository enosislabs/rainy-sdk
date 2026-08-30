//! OpenAI-compatible embeddings example.

use rainy_sdk::{EmbeddingEncodingFormat, EmbeddingsRequest, RainyClient};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = RainyClient::with_api_key(
        std::env::var("RAINY_API_KEY").unwrap_or_else(|_| "your-api-key-here".to_string()),
    )?;
    let request = EmbeddingsRequest::text("compatible/embedding-model", "Rust SDK")
        .with_encoding_format(EmbeddingEncodingFormat::Float)
        .with_dimensions(768);
    let response = client.create_embeddings(request).await?;
    println!("received {} embedding(s)", response.data.len());
    Ok(())
}
