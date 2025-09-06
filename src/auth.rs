use crate::error::{RainyError, Result};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
use std::time::Duration;

/// Manages authentication and configuration for the Rainy API client.
///
/// This struct holds all the necessary configuration for connecting to the Rainy API,
/// including the API key, base URL, timeout settings, and retry logic. It provides
/// a builder pattern for easy and readable configuration.
///
/// # Examples
///
/// ```
/// # use rainy_sdk::auth::AuthConfig;
/// let config = AuthConfig::new("ra-your-api-key")
///     .with_base_url("https://api.example.com")
///     .with_timeout(60)
///     .with_max_retries(5)
///     .with_retry(true);
///
/// assert_eq!(config.timeout_seconds, 60);
/// assert_eq!(config.max_retries, 5);
/// ```
#[derive(Debug, Clone)]
pub struct AuthConfig {
    /// The API key used for authenticating with the Rainy API.
    ///
    /// This key is sent in the `Authorization` header of every request.
    /// It must start with the prefix `ra-`.
    pub api_key: String,

    /// The base URL for the Rainy API.
    ///
    /// Defaults to `https://api.enosislabs.com`. This can be overridden for testing
    /// or for connecting to a self-hosted instance of the Rainy API.
    pub base_url: String,

    /// The timeout for HTTP requests, in seconds.
    ///
    /// This is the maximum amount of time the client will wait for a response
    /// from the server. Defaults to 30 seconds.
    pub timeout_seconds: u64,

    /// The maximum number of retry attempts for failed requests.
    ///
    /// This setting applies only when `enable_retry` is `true`. Defaults to 3.
    pub max_retries: u32,

    /// A flag to enable or disable automatic retries with exponential backoff.
    ///
    /// When `true`, the client will automatically retry failed requests that are
    /// deemed retryable (e.g., network errors, server-side 5xx errors).
    /// Defaults to `true`.
    pub enable_retry: bool,

    /// The user agent string sent with each request.
    ///
    /// Defaults to `rainy-sdk-rust/{version}`.
    pub user_agent: String,
}

impl AuthConfig {
    /// Creates a new `AuthConfig` with the given API key and default settings.
    ///
    /// This is the primary way to start building a client configuration.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your Rainy API key. It is automatically converted into a `String`.
    ///
    /// # Returns
    ///
    /// A new `AuthConfig` instance with default values for base URL, timeout, etc.
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

    /// Sets a custom base URL for the API.
    ///
    /// Use this to connect to a different API endpoint, such as a staging server
    /// or a self-hosted instance.
    ///
    /// # Arguments
    ///
    /// * `base_url` - The new base URL to use.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Sets the request timeout in seconds.
    ///
    /// # Arguments
    ///
    /// * `seconds` - The timeout duration in seconds.
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Sets the maximum number of retry attempts.
    ///
    /// This only has an effect if retries are enabled.
    ///
    /// # Arguments
    ///
    /// * `retries` - The maximum number of times to retry a failed request.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Enables or disables automatic retries.
    ///
    /// # Arguments
    ///
    /// * `enable` - Set to `true` to enable retries, `false` to disable.
    pub fn with_retry(mut self, enable: bool) -> Self {
        self.enable_retry = enable;
        self
    }

    /// Sets a custom user agent string for all requests.
    ///
    /// # Arguments
    ///
    /// * `user_agent` - The custom user agent string.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Validates the `AuthConfig` to ensure it is correctly configured.
    ///
    /// This method checks for:
    /// - A non-empty API key.
    /// - The correct API key format (must start with `ra-`).
    /// - A valid base URL format.
    ///
    /// # Returns
    ///
    /// * `Ok(())` if the configuration is valid.
    /// * `Err(RainyError)` if validation fails.
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

    /// Builds the set of HTTP headers required for API requests.
    ///
    /// This includes the `Authorization`, `User-Agent`, and `Content-Type` headers.
    ///
    /// # Returns
    ///
    /// A `reqwest::header::HeaderMap` containing the necessary headers, or an
    /// error if the headers could not be created.
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

    /// Returns the request timeout as a `std::time::Duration`.
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

/// A simple, legacy rate limiter.
///
/// This implementation is basic and not recommended for production use.
/// It is kept for backward compatibility. The preferred way to handle rate
/// limiting is by enabling the `rate-limiting` feature, which uses a more
/// robust `governor`-based implementation in the `RainyClient`.
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
    /// * `requests_per_minute` - The number of requests allowed per minute.
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            last_request: std::time::Instant::now(),
            request_count: 0,
        }
    }

    /// Pauses execution if the rate limit has been exceeded.
    ///
    /// This async function will block until the next request is allowed.
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
