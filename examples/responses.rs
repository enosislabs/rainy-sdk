//! OpenAI-compatible Responses example with reasoning and a function tool.

use rainy_sdk::{RainyClient, ResponsesRequest};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = RainyClient::with_api_key(
        std::env::var("RAINY_API_KEY").unwrap_or_else(|_| "your-api-key-here".to_string()),
    )?;
    let request = ResponsesRequest::text(
        "compatible/responses-model",
        "What is the capital of France?",
    )
    .with_reasoning_effort("medium")
    .with_max_output_tokens(256)
    .add_function_tool(
        "lookup",
        "Look up a value in an application-owned data source.",
        rainy_sdk::serde_json::json!({
            "type": "object",
            "properties": {"key": {"type": "string"}},
            "required": ["key"],
            "additionalProperties": false
        }),
    );

    let response = client.create_response(request).await?.0;
    println!("{}", response.text().unwrap_or_default());
    Ok(())
}
