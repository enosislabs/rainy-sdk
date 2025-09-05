use rainy_sdk::{AuthConfig, RainyError};
use std::time::Duration;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_config_validation() {
        // Test missing keys
        let config = AuthConfig::new();
        assert!(config.validate().is_err());

        // Test valid config with API key
        let config = AuthConfig::new()
            .with_api_key("test-key")
            .with_base_url("https://api.example.com");
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_auth_config_with_timeout() {
        let config = AuthConfig::new()
            .with_api_key("test-key")
            .with_timeout(Duration::from_secs(60));

        assert_eq!(config.timeout, Duration::from_secs(60));
    }

    #[test]
    fn test_error_retryable() {
        let auth_error = RainyError::Authentication {
            message: "Invalid key".to_string(),
        };
        assert!(!auth_error.is_retryable());

        // The following test is commented out because creating a reqwest::Error
        // of a specific kind for testing purposes is not straightforward.
        // let timeout_error = RainyError::Http(reqwest::Error::from(
        //     std::io::Error::new(std::io::ErrorKind::TimedOut, "timeout")
        // ));
        // assert!(timeout_error.is_retryable());
    }
}
