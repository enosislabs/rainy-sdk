use crate::error::{RainyError, Result};
use futures::StreamExt;
use reqwest::Response;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT};
use secrecy::{ExposeSecret, SecretString};
use serde::Serialize;
use std::time::Duration;

/// Maximum request body accepted by ordinary Rainy API routes.
pub(crate) const GENERAL_REQUEST_BODY_BYTES: usize = 1_048_576;
/// Maximum request body accepted by model-generation and embedding routes.
pub(crate) const MODEL_REQUEST_BODY_BYTES: usize = 33_554_432;
/// Maximum response body retained by the SDK for a normal JSON response.
pub(crate) const MAX_RESPONSE_BODY_BYTES: usize = 16 * 1024 * 1024;
/// Maximum error body retained by the SDK. Error payloads are not useful after
/// a small bounded prefix and must never become an unbounded memory sink.
pub(crate) const MAX_ERROR_BODY_BYTES: usize = 64 * 1024;

/// Serializes a JSON request body while enforcing the corresponding Rainy
/// route limit before a request is sent.
pub(crate) fn serialize_json_body<T: Serialize>(value: &T, max_bytes: usize) -> Result<Vec<u8>> {
    let body = serde_json::to_vec(value).map_err(|error| RainyError::Serialization {
        message: "Failed to serialize JSON request body".to_string(),
        source_error: Some(error.to_string()),
    })?;
    if body.len() > max_bytes {
        return Err(RainyError::PayloadTooLarge {
            message: "JSON request body exceeds the configured safety limit".to_string(),
            max_bytes,
        });
    }
    Ok(body)
}

/// Parses the numeric form of the HTTP `Retry-After` header.
///
/// Rainy's rate-limit responses use seconds.  Invalid, fractional, negative,
/// and unreasonably large values are ignored instead of being allowed to turn
/// a retry into an unbounded sleep.  HTTP-date values are deliberately not
/// guessed at here: honoring a malformed date would be less safe than using
/// the SDK's bounded exponential backoff.
pub(crate) fn retry_after_seconds(headers: &HeaderMap) -> Option<u64> {
    headers
        .get("retry-after")
        .and_then(|header| header.to_str().ok())
        .and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|seconds| *seconds <= 86_400)
}

/// Read an HTTP body without allowing an untrusted peer to force an
/// unbounded allocation.
pub(crate) async fn read_limited_response_body(
    response: Response,
    max_bytes: usize,
) -> Result<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|length| length > max_bytes as u64)
    {
        return Err(RainyError::PayloadTooLarge {
            message: "HTTP response body exceeds the configured safety limit".to_string(),
            max_bytes,
        });
    }

    let mut body = Vec::new();
    let mut chunks = response.bytes_stream();
    while let Some(chunk) = chunks.next().await {
        let chunk = chunk.map_err(|_error| RainyError::Network {
            message: "Failed while reading HTTP response body".to_string(),
            retryable: false,
            source_error: None,
        })?;
        if body.len().saturating_add(chunk.len()) > max_bytes {
            return Err(RainyError::PayloadTooLarge {
                message: "HTTP response body exceeds the configured safety limit".to_string(),
                max_bytes,
            });
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

/// Validate a configured service URL before it is used for authenticated HTTP.
///
/// HTTPS is required for real hosts. Plain HTTP is allowed only for loopback
/// test servers, which keeps local integration tests useful without allowing a
/// caller to accidentally send credentials over the network in cleartext.
pub(crate) fn validate_service_url(raw: &str, code: &str) -> Result<url::Url> {
    if raw != raw.trim() {
        return Err(RainyError::InvalidRequest {
            code: code.to_string(),
            message: "Base URL must not contain surrounding whitespace".to_string(),
            details: None,
        });
    }

    let parsed = url::Url::parse(raw).map_err(|_| RainyError::InvalidRequest {
        code: code.to_string(),
        message: "Base URL is not a valid URL".to_string(),
        details: None,
    })?;

    let host = parsed
        .host_str()
        .ok_or_else(|| RainyError::InvalidRequest {
            code: code.to_string(),
            message: "Base URL must include a host".to_string(),
            details: None,
        })?;
    let loopback = host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|address| address.is_loopback());
    if parsed.scheme() != "https" && !(parsed.scheme() == "http" && loopback) {
        return Err(RainyError::InvalidRequest {
            code: code.to_string(),
            message: "Base URL must use HTTPS (HTTP is allowed only for loopback hosts)"
                .to_string(),
            details: None,
        });
    }

    if !parsed.username().is_empty()
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(RainyError::InvalidRequest {
            code: code.to_string(),
            message: "Base URL must not contain credentials, a query, or a fragment".to_string(),
            details: None,
        });
    }

    Ok(parsed)
}

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
///     .with_api_base_url("https://api.example.com/openai/v1")
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
    pub api_key: SecretString,

    /// The base URL of the Rainy API. Defaults to the official endpoint.
    pub base_url: String,

    /// Optional base URL for versioned API endpoints.
    ///
    /// When unset, endpoints use `{base_url}/api/v1`. Set this when a compatible
    /// deployment exposes the API under another prefix, such as `/v1`.
    pub api_base_url: Option<String>,

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
            api_key: SecretString::from(api_key.into()),
            base_url: crate::DEFAULT_BASE_URL.to_string(),
            api_base_url: None,
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

    /// Sets the complete base URL used for versioned API endpoints.
    ///
    /// Endpoint paths are appended directly to this value. Root-level routes,
    /// such as `/health`, continue to use [`Self::with_base_url`].
    pub fn with_api_base_url(mut self, api_base_url: impl Into<String>) -> Self {
        self.api_base_url = Some(api_base_url.into());
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
    /// Supports standard keys (`ra-` plus 48 characters) and platform keys
    /// (`rk_live_` plus 48 characters), matching the Rainy API.
    ///
    /// # Returns
    ///
    /// A `Result` that is `Ok(())` if the configuration is valid, or a `RainyError` if it's not.
    pub fn validate(&self) -> Result<()> {
        if self.api_key.expose_secret().is_empty() {
            return Err(RainyError::Authentication {
                code: "EMPTY_API_KEY".to_string(),
                message: "API key cannot be empty".to_string(),
                retryable: false,
            });
        }

        let key = self.api_key.expose_secret();

        let valid_length = (key.starts_with("ra-") && key.len() == 51)
            || (key.starts_with("rk_live_") && key.len() == 56);
        if !valid_length || !key.is_ascii() || key.bytes().any(|byte| byte.is_ascii_whitespace()) {
            return Err(RainyError::Authentication {
                code: "INVALID_API_KEY_FORMAT".to_string(),
                message: "API key has an invalid prefix or length".to_string(),
                retryable: false,
            });
        }

        if self.timeout_seconds == 0 {
            return Err(RainyError::InvalidRequest {
                code: "INVALID_TIMEOUT".to_string(),
                message: "Request timeout must be greater than zero".to_string(),
                details: None,
            });
        }

        // Validate URL format
        validate_service_url(&self.base_url, "INVALID_BASE_URL")?;

        if let Some(api_base_url) = self.api_base_url.as_ref() {
            validate_service_url(api_base_url, "INVALID_API_BASE_URL")?;
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
        let auth_value = format!("Bearer {}", self.api_key.expose_secret());
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
