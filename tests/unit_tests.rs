use rainy_sdk::models::model_constants::*;
use rainy_sdk::{
    AuthConfig, ChatCompletionRequest, ChatMessage, MessageRole, RainyError, RetryConfig,
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_validation() {
        // Test valid API key format
        let config = AuthConfig::new("ra-test-key");
        assert!(config.validate().is_ok());

        // Test invalid API key format
        let config = AuthConfig::new("invalid-key");
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_auth_config_builder() {
        let config = AuthConfig::new("ra-test-key")
            .with_timeout(60)
            .with_max_retries(5);

        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.max_retries, 5);
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_chat_message_creation() {
        let user_msg = ChatMessage::user("Hello");
        assert_eq!(user_msg.role, MessageRole::User);
        assert_eq!(user_msg.content, "Hello");

        let system_msg = ChatMessage::system("You are helpful");
        assert_eq!(system_msg.role, MessageRole::System);
        assert_eq!(system_msg.content, "You are helpful");

        let assistant_msg = ChatMessage::assistant("Hi there");
        assert_eq!(assistant_msg.role, MessageRole::Assistant);
        assert_eq!(assistant_msg.content, "Hi there");
    }

    #[test]
    fn test_chat_completion_request_builder() {
        let messages = vec![ChatMessage::user("Test message")];
        let request = ChatCompletionRequest::new(OPENAI_GPT_4O, messages.clone())
            .with_temperature(0.7)
            .with_max_tokens(100)
            .with_user("test-user");

        assert_eq!(request.model, OPENAI_GPT_4O);
        assert_eq!(request.messages, messages);
        assert_eq!(request.temperature, Some(0.7));
        assert_eq!(request.max_tokens, Some(100));
        assert_eq!(request.user, Some("test-user".to_string()));
    }

    #[test]
    fn test_retry_config() {
        let config = RetryConfig::new(5);
        assert_eq!(config.max_retries, 5);

        // Test delay calculation
        let delay0 = config.delay_for_attempt(0);
        let delay1 = config.delay_for_attempt(1);
        let delay2 = config.delay_for_attempt(2);

        assert!(delay1.as_millis() >= delay0.as_millis());
        assert!(delay2.as_millis() >= delay1.as_millis());
        assert!(delay2.as_millis() <= config.max_delay_ms as u128);
    }

    #[test]
    fn test_error_retryability() {
        let auth_error = RainyError::Authentication {
            code: "INVALID_KEY".to_string(),
            message: "Invalid key".to_string(),
            retryable: false,
        };
        assert!(!auth_error.is_retryable());

        let network_error = RainyError::Network {
            message: "Connection failed".to_string(),
            retryable: true,
            source_error: None,
        };
        assert!(network_error.is_retryable());

        let rate_limit_error = RainyError::RateLimit {
            code: "RATE_LIMIT_EXCEEDED".to_string(),
            message: "Too many requests".to_string(),
            retry_after: Some(60),
            current_usage: None,
        };
        assert!(rate_limit_error.is_retryable());
        assert_eq!(rate_limit_error.retry_after(), Some(60));
    }

    #[test]
    fn test_error_codes() {
        let auth_error = RainyError::Authentication {
            code: "INVALID_KEY".to_string(),
            message: "Invalid key".to_string(),
            retryable: false,
        };
        assert_eq!(auth_error.code(), Some("INVALID_KEY"));

        let network_error = RainyError::Network {
            message: "Connection failed".to_string(),
            retryable: true,
            source_error: None,
        };
        assert_eq!(network_error.code(), None);
    }

    #[test]
    fn test_model_constants() {
        // Test new provider-prefixed constants
        assert_eq!(OPENAI_GPT_4O, "gpt-4o");
        assert_eq!(GOOGLE_GEMINI_2_5_PRO, "gemini-2.5-pro");
        assert_eq!(GROQ_LLAMA_3_1_8B_INSTANT, "llama-3.1-8b-instant");
        assert_eq!(CEREBRAS_LLAMA3_1_8B, "cerebras/llama3.1-8b");

        // Test legacy constants (deprecated but still available)
        assert_eq!(OPENAI_GPT_4O, "gpt-4o");
        assert_eq!(GOOGLE_GEMINI_2_5_PRO, "gemini-2.5-pro");
        assert_eq!(GROQ_LLAMA_3_1_8B_INSTANT, "llama-3.1-8b-instant");
    }
}
