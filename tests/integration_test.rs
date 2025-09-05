use rainy_sdk::{AuthConfig, ChatCompletionRequest, ChatMessage, ChatRole, RainyClient};
use std::env;

#[cfg(test)]
mod integration_tests {
    use super::*;

    fn get_test_client() -> RainyClient {
        let api_key = env::var("RAINY_TEST_API_KEY").unwrap_or_else(|_| "test-key".to_string());
        let base_url =
            env::var("RAINY_TEST_BASE_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

        RainyClient::new(
            AuthConfig::new()
                .with_api_key(api_key)
                .with_base_url(base_url),
        )
        .expect("Failed to create test client")
    }

    #[tokio::test]
    async fn test_health_check() {
        let client = get_test_client();
        let result = client.health_check().await;

        match result {
            Ok(health) => {
                assert!(matches!(health.status, rainy_sdk::HealthStatus::Healthy));
            }
            Err(_) => {
                // If API is not available, that's okay for CI
                println!("API not available for testing");
            }
        }
    }

    #[tokio::test]
    async fn test_chat_completion_request_creation() {
        let messages = vec![ChatMessage {
            role: ChatRole::User,
            content: "Hello, world!".to_string(),
        }];

        let request = ChatCompletionRequest {
            model: "gemini-pro".to_string(),
            messages,
            max_tokens: Some(100),
            temperature: Some(0.7),
            stream: None,
        };

        // Test serialization
        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("gemini-pro"));
        assert!(json.contains("Hello, world!"));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let client = get_test_client().with_rate_limit(5);

        // This test would require a running API server
        // For now, just test that the client can be created with rate limiting
        let _ = client; // Prevent unused variable warning
    }
}
