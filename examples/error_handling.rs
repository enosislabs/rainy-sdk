//! Handle typed errors and retry only an explicitly safe discovery operation.

use rainy_sdk::{
    ChatCompletionRequest, ChatMessage, RainyClient, RainyError, RetryConfig, retry_with_backoff,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key =
        std::env::var("RAINY_API_KEY").unwrap_or_else(|_| "your-api-key-here".to_string());
    let client = RainyClient::with_api_key(api_key)?;

    let request = ChatCompletionRequest::new(
        "compatible/chat-model",
        vec![ChatMessage::user("Say hello in one sentence.")],
    );

    match client.chat_completion(request).await {
        Ok((response, metadata)) => {
            println!("{}", response.choices[0].message.content);
            println!("request id: {:?}", metadata.request_id);
        }
        Err(RainyError::Authentication { code, message, .. }) => {
            eprintln!("authentication failed ({code}): {message}");
        }
        Err(RainyError::RateLimit {
            message,
            retry_after,
            ..
        }) => {
            eprintln!("rate limited: {message}; retry-after={retry_after:?}");
        }
        Err(error) if error.is_retryable() => {
            eprintln!("transient service error: {error}");
        }
        Err(error) => eprintln!("request failed: {error}"),
    }

    // Model discovery is safe to retry. Inference POSTs are deliberately not
    // wrapped in this helper because replaying them can duplicate work.
    let retry_config = RetryConfig::new(2);
    match retry_with_backoff(&retry_config, || client.list_models()).await {
        Ok(models) => println!("discovered {} model(s)", models.data.len()),
        Err(error) => eprintln!("model discovery failed: {error}"),
    }

    Ok(())
}
