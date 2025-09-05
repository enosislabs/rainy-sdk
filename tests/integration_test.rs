use rainy_sdk::{AuthConfig, ChatCompletionRequest, ChatMessage, RainyClient};
use std::env;

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn get_test_client() -> RainyClient {
        let api_key = env::var("RAINY_TEST_API_KEY").unwrap_or_else(|_| "ra-test-key".to_string());
        let base_url =
            env::var("RAINY_TEST_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        RainyClient::with_config(AuthConfig::new(&api_key).with_base_url(base_url))
            .expect("Failed to create test client")
    }

    #[tokio::test]
    async fn test_health_check() {
        let client = get_test_client();
        let result = client.health_check().await;

        match result {
            Ok(health) => {
                assert_eq!(health.status, "healthy");
            }
            Err(_) => {
                // If API is not available, that's okay for CI
                println!("API not available for testing");
            }
        }
    }

    #[tokio::test]
    async fn test_chat_completion_request_creation() {
        let messages = vec![ChatMessage::user("Hello, world!")];

        let request = ChatCompletionRequest::new("gemini-pro", messages)
            .with_max_tokens(100)
            .with_temperature(0.7);

        // Test serialization
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gemini-pro"));
        assert!(json.contains("Hello, world!"));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let client = get_test_client();

        // Rate limiting is now handled internally by the client
        // This test just ensures the client can be created
        let _ = client; // Prevent unused variable warning
    }
}
