use crate::error::{RainyError, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use std::time::Duration;

/// Configuration for authentication and client behavior.
///
/// `AuthConfig` holds all the necessary information for authenticating with the Rainy API,
/// as well as settings for request behavior like timeouts and retries.
///
/// # Examples
///
/// ```rust
/// use rainy_sdk::auth::AuthConfig;
///
/// let config = AuthConfig::new("your-api-key")
///     .with_base_url("https://api.example.com")
///     .with_timeout(60)
///     .with_max_retries(5);
///
/// assert_eq!(config.base_url, "https://api.example.com");
/// assert_eq!(config.timeout_seconds, 60);
/// assert_eq!(config.max_retries, 5);
/// ```
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// The API key used for authenticating with the Rainy API.
    pub api_key: String,

    /// The base URL of the Rainy API. Defaults to the official endpoint.
    pub base_url: String,

    /// The timeout for HTTP requests, in seconds.
    pub timeout_seconds: u64,

    /// The maximum number of times to retry a failed request.
    pub max_retries: u32,

    /// A flag to enable or disable automatic retries with exponential backoff.
    pub enable_retry: bool,

    /// The user agent string to send with each request.
    pub user_agent: String,
}

impl AuthConfig {
    /// Creates a new `AuthConfig` with the given API key and default settings.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your Rainy API key.
    pub fn new(api_key: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            base_url: crate::DEFAULT_BASE_URL.to_string(),
            timeout_seconds: 30,
            max_retries: 3,
            enable_retry: true,
            user_agent: format!("rainy-sdk/{}", crate::VERSION),
        }
    }

    /// Sets a custom base URL for the API.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The new base URL to use.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Sets a custom timeout for HTTP requests.
    ///
    /// # Arguments
    ///
    /// * `seconds` - The timeout duration in seconds.
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Sets the maximum number of retry attempts for failed requests.
    ///
    /// # Arguments
    ///
    /// * `retries` - The maximum number of retries.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Enables or disables automatic retries.
    ///
    /// # Arguments
    ///
    /// * `enable` - `true` to enable retries, `false` to disable.
    pub fn with_retry(mut self, enable: bool) -> Self {
        self.enable_retry = enable;
        self
    }

    /// Sets a custom user agent string for requests.
    ///
    /// # Arguments
    ///
    /// * `user_agent` - The new user agent string.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Validates the `AuthConfig` settings.
    ///
    /// This method checks for common configuration errors, such as an empty API key
    /// or an invalid base URL.
    ///
    /// # Returns
    ///
    /// A `Result` that is `Ok(())` if the configuration is valid, or a `RainyError` if it's not.
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

    /// Builds the necessary HTTP headers for an API request.
    ///
    /// This method constructs a `HeaderMap` containing the `Authorization` and `User-Agent`
    /// headers based on the `AuthConfig`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `HeaderMap` or a `RainyError` if header creation fails.
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

    /// Returns the request timeout as a `Duration`.
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

/// A simple rate limiter.
///
/// This rate limiter is deprecated and should not be used in new code.
/// The `RainyClient` now uses a more robust, feature-flagged rate limiting mechanism
/// based on the `governor` crate.
#[deprecated(note = "Use the governor-based rate limiting in RainyClient instead")]
#[derive(Debug)]
pub struct RateLimiter {
    requests_per_minute: u32,
    last_request: std::time::Instant,
    request_count: u32,
}

#[allow(deprecated)]
impl RateLimiter {
    /// Creates a new `RateLimiter`.
    ///
    /// # Arguments
    ///
    /// * `requests_per_minute` - The maximum number of requests allowed per minute.
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            last_request: std::time::Instant::now(),
            request_count: 0,
        }
    }

    /// Pauses execution if the rate limit has been exceeded.
    ///
    /// This method will asynchronously wait until the next request can be sent without
    /// violating the rate limit.
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
