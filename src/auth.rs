use crate::error::{RainyError, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use std::time::Duration;

/// Authentication configuration for the Rainy API
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// API key for authentication
    pub api_key: String,

    /// Base URL for the API (defaults to official endpoint)
    pub base_url: String,

    /// Request timeout in seconds
    pub timeout_seconds: u64,

    /// Maximum number of retry attempts
    pub max_retries: u32,

    /// Enable automatic retry with exponential backoff
    pub enable_retry: bool,

    /// User agent string for requests
    pub user_agent: String,
}

impl AuthConfig {
    /// Create a new auth config with an API key
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: crate::DEFAULT_BASE_URL.to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            enable_retry: true,
            user_agent: format!("rainy-sdk-rust/{}", crate::VERSION),
        }
    }

    /// Set custom base URL
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Set request timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Set maximum retry attempts
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Enable or disable automatic retries
    pub fn with_retry(mut self, enable: bool) -> Self {
        self.enable_retry = enable;
        self
    }

    /// Set custom user agent
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Validate the API key format
    pub fn validate(&self) -> Result<()> {
        if self.api_key.is_empty() {
            return Err(RainyError::Authentication {
                code: "EMPTY_API_KEY".to_string(),
                message: "API key cannot be empty".to_string(),
                retryable: false,
            });
        }

        // Basic API key format validation (starts with 'ra-')
        if !self.api_key.starts_with("ra-") {
            return Err(RainyError::Authentication {
                code: "INVALID_API_KEY_FORMAT".to_string(),
                message: "API key must start with 'ra-'".to_string(),
                retryable: false,
            });
        }

        // Validate URL format
        if url::Url::parse(&self.base_url).is_err() {
            return Err(RainyError::InvalidRequest {
                code: "INVALID_BASE_URL".to_string(),
                message: "Base URL is not a valid URL".to_string(),
                details: None,
            });
        }

        Ok(())
    }

    /// Build headers for HTTP requests
    pub fn build_headers(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();

        // Set User-Agent
        headers.insert(USER_AGENT, HeaderValue::from_str(&self.user_agent)?);

        // Set Content-Type for JSON requests
        headers.insert(
            reqwest::header::CONTENT_TYPE,
            HeaderValue::from_static("application/json"),
        );

        // Set authorization header
        let auth_value = format!("Bearer {}", self.api_key);
        headers.insert(AUTHORIZATION, HeaderValue::from_str(&auth_value)?);

        Ok(headers)
    }

    /// Get timeout duration
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }
}

impl std::fmt::Display for AuthConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "AuthConfig {{ base_url: {}, timeout: {}s, retries: {} }}",
            self.base_url, self.timeout_seconds, self.max_retries
        )
    }
}

// Legacy rate limiter - kept for backward compatibility but marked deprecated
#[deprecated(note = "Use the governor-based rate limiting in RainyClient instead")]
#[derive(Debug)]
pub struct RateLimiter {
    requests_per_minute: u32,
    last_request: std::time::Instant,
    request_count: u32,
}

#[allow(deprecated)]
impl RateLimiter {
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            last_request: std::time::Instant::now(),
            request_count: 0,
        }
    }

    pub async fn wait_if_needed(&mut self) -> Result<()> {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_request);

        // Reset counter if a minute has passed
        if elapsed >= Duration::from_secs(60) {
            self.request_count = 0;
            self.last_request = now;
        }

        // Check if we've exceeded the rate limit
        if self.request_count >= self.requests_per_minute {
            let wait_time = Duration::from_secs(60) - elapsed;
            tokio::time::sleep(wait_time).await;
            self.request_count = 0;
            self.last_request = std::time::Instant::now();
        }

        self.request_count += 1;
        Ok(())
    }
}
