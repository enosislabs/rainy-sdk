//! Secret-aware authentication and bounded HTTP-body helpers.

use crate::error::{RainyError, Result};
use futures::StreamExt;
use reqwest::Response;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT};
use secrecy::{ExposeSecret, SecretString};
use std::{fmt, time::Duration};
use url::Url;

/// Maximum body size for general non-inference JSON requests.
pub(crate) const GENERAL_REQUEST_BODY_BYTES: usize = 1024 * 1024;
/// Maximum body size for one inference request.
pub(crate) const MODEL_REQUEST_BODY_BYTES: usize = 8 * 1024 * 1024;
/// Maximum successful response body retained in memory.
pub(crate) const MAX_RESPONSE_BODY_BYTES: usize = 8 * 1024 * 1024;
/// Maximum error body retained for diagnostics.
pub(crate) const MAX_ERROR_BODY_BYTES: usize = 64 * 1024;
/// Maximum accepted `Retry-After` delay.
const MAX_RETRY_AFTER_SECONDS: u64 = 86_400;

/// Authentication convention used by a protocol request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AuthScheme {
    /// `Authorization: Bearer <key>`.
    Bearer,
    /// `x-api-key: <key>`.
    XApiKey,
}

/// Configuration for API-key authenticated protocol clients.
///
/// The API key is held as a [`SecretString`] and is never included in the
/// configuration's `Debug` or `Display` output. Key format validation is
/// intentionally protocol-neutral so the same client can target Rainy or a
/// compatible OpenAI/Anthropic endpoint.
#[derive(Clone)]
pub struct AuthConfig {
    /// Secret used for protocol authentication.
    pub api_key: SecretString,
    /// Host base URL used by root-level routes.
    pub base_url: String,
    /// Explicit versioned API base URL. If unset, `/api/v1` is appended to the
    /// base URL unless the base URL already has a non-root path.
    pub api_base_url: Option<String>,
    /// Request timeout in seconds.
    pub timeout_seconds: u64,
    /// Maximum retries for explicitly safe operations.
    pub max_retries: u32,
    /// Whether safe operations may retry.
    pub enable_retry: bool,
    /// User-Agent header.
    pub user_agent: String,
}

impl AuthConfig {
    /// Creates a configuration with the Rainy service as the default backend.
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

    /// Sets the root base URL.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Sets the exact base URL for versioned protocol routes.
    pub fn with_api_base_url(mut self, api_base_url: impl Into<String>) -> Self {
        self.api_base_url = Some(api_base_url.into());
        self
    }

    /// Sets the request timeout in seconds.
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Sets the retry count for safe operations.
    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// Enables or disables retries for safe operations.
    pub fn with_retry(mut self, enable: bool) -> Self {
        self.enable_retry = enable;
        self
    }

    /// Sets the User-Agent header.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }

    /// Validates credentials, URLs, and header-safe configuration.
    pub fn validate(&self) -> Result<()> {
        let key = self.api_key.expose_secret();
        if key.is_empty() {
            return Err(RainyError::Authentication {
                code: "EMPTY_API_KEY".to_string(),
                message: "API key cannot be empty".to_string(),
                retryable: false,
            });
        }
        if key.len() > 4096
            || key
                .chars()
                .any(|character| character.is_whitespace() || character.is_control())
        {
            return Err(RainyError::Authentication {
                code: "INVALID_API_KEY".to_string(),
                message: "API key contains invalid characters".to_string(),
                retryable: false,
            });
        }
        validate_service_url(&self.base_url, "INVALID_BASE_URL")?;
        if let Some(api_base_url) = &self.api_base_url {
            validate_service_url(api_base_url, "INVALID_API_BASE_URL")?;
        }
        HeaderValue::from_str(&self.user_agent).map_err(|_| RainyError::InvalidRequest {
            code: "INVALID_USER_AGENT".to_string(),
            message: "User-Agent contains invalid header characters".to_string(),
            details: None,
        })?;
        Ok(())
    }

    /// Builds Bearer headers for OpenAI-compatible requests.
    pub fn build_headers(&self) -> Result<HeaderMap> {
        self.validate()?;
        self.build_protocol_headers(AuthScheme::Bearer, None)
    }

    /// Builds native Anthropic headers for Messages requests.
    pub fn build_anthropic_headers(&self, version: &str) -> Result<HeaderMap> {
        self.validate()?;
        self.build_protocol_headers(AuthScheme::XApiKey, Some(version))
    }

    pub(crate) fn build_protocol_headers(
        &self,
        scheme: AuthScheme,
        anthropic_version: Option<&str>,
    ) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&self.user_agent).map_err(|_| RainyError::InvalidRequest {
                code: "INVALID_USER_AGENT".to_string(),
                message: "User-Agent contains invalid header characters".to_string(),
                details: None,
            })?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

        match scheme {
            AuthScheme::Bearer => {
                let value = format!("Bearer {}", self.api_key.expose_secret());
                headers.insert(
                    AUTHORIZATION,
                    HeaderValue::from_str(&value).map_err(|_| RainyError::Authentication {
                        code: "INVALID_API_KEY".to_string(),
                        message: "API key cannot be used in an HTTP header".to_string(),
                        retryable: false,
                    })?,
                );
            }
            AuthScheme::XApiKey => {
                headers.insert(
                    "x-api-key",
                    HeaderValue::from_str(self.api_key.expose_secret()).map_err(|_| {
                        RainyError::Authentication {
                            code: "INVALID_API_KEY".to_string(),
                            message: "API key cannot be used in an HTTP header".to_string(),
                            retryable: false,
                        }
                    })?,
                );
                let version = anthropic_version.ok_or_else(|| RainyError::InvalidRequest {
                    code: "MISSING_ANTHROPIC_VERSION".to_string(),
                    message: "Anthropic requests require an API version".to_string(),
                    details: None,
                })?;
                headers.insert(
                    "anthropic-version",
                    HeaderValue::from_str(version).map_err(|_| RainyError::InvalidRequest {
                        code: "INVALID_ANTHROPIC_VERSION".to_string(),
                        message: "Anthropic API version is not a valid header value".to_string(),
                        details: None,
                    })?,
                );
            }
        }
        Ok(headers)
    }

    /// Returns the configured timeout.
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_seconds)
    }
}

impl fmt::Debug for AuthConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AuthConfig")
            .field("api_key", &"<redacted>")
            .field("base_url", &safe_url_label(&self.base_url))
            .field(
                "api_base_url",
                &self.api_base_url.as_deref().map(safe_url_label),
            )
            .field("timeout_seconds", &self.timeout_seconds)
            .field("max_retries", &self.max_retries)
            .field("enable_retry", &self.enable_retry)
            .field("user_agent", &self.user_agent)
            .finish()
    }
}

impl fmt::Display for AuthConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "AuthConfig {{ base_url: {}, timeout: {}s, retries: {} }}",
            safe_url_label(&self.base_url),
            self.timeout_seconds,
            self.max_retries
        )
    }
}

/// Returns a host-only URL label suitable for diagnostics.
pub(crate) fn safe_url_label(value: &str) -> String {
    let Ok(url) = Url::parse(value) else {
        return "<invalid-url>".to_string();
    };
    match url.host_str() {
        Some(host) => format!("{}://{}", url.scheme(), host),
        None => "<invalid-url>".to_string(),
    }
}

/// Validates a configured service URL without logging or normalizing secrets.
pub(crate) fn validate_service_url(value: &str, code: &str) -> Result<Url> {
    if value.trim() != value || value.is_empty() {
        return Err(RainyError::InvalidRequest {
            code: code.to_string(),
            message: "service URL must not be empty or surrounded by whitespace".to_string(),
            details: None,
        });
    }
    let url = Url::parse(value).map_err(|_| RainyError::InvalidRequest {
        code: code.to_string(),
        message: "service URL is not valid".to_string(),
        details: None,
    })?;
    let host = url.host_str().ok_or_else(|| RainyError::InvalidRequest {
        code: code.to_string(),
        message: "service URL must include a host".to_string(),
        details: None,
    })?;
    if url.username() != ""
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(RainyError::InvalidRequest {
            code: code.to_string(),
            message: "service URL must not contain credentials, query parameters, or fragments"
                .to_string(),
            details: None,
        });
    }

    match url.scheme() {
        "https" => {}
        "http" if is_loopback_host(host) => {}
        _ => {
            return Err(RainyError::InvalidRequest {
                code: code.to_string(),
                message: "service URL must use HTTPS; HTTP is allowed only for loopback testing"
                    .to_string(),
                details: None,
            });
        }
    }
    Ok(url)
}

fn is_loopback_host(host: &str) -> bool {
    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<std::net::IpAddr>()
            .is_ok_and(|address| address.is_loopback())
}

/// Serializes a request body while enforcing a byte limit before sending it.
pub(crate) fn serialize_json_body<T: serde::Serialize>(body: &T, limit: usize) -> Result<Vec<u8>> {
    let bytes = serde_json::to_vec(body).map_err(|error| RainyError::Serialization {
        message: "failed to serialize JSON request".to_string(),
        source_error: Some(error.to_string()),
    })?;
    if bytes.len() > limit {
        return Err(RainyError::PayloadTooLarge {
            message: "JSON request body exceeds the configured safety limit".to_string(),
            max_bytes: limit,
        });
    }
    Ok(bytes)
}

/// Reads a response body into bounded memory.
pub(crate) async fn read_limited_response_body(
    response: Response,
    limit: usize,
) -> Result<Vec<u8>> {
    if response
        .content_length()
        .is_some_and(|length| length > limit as u64)
    {
        return Err(RainyError::PayloadTooLarge {
            message: "response body exceeds the configured safety limit".to_string(),
            max_bytes: limit,
        });
    }

    let mut body = Vec::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(RainyError::from)?;
        if body.len().saturating_add(chunk.len()) > limit {
            return Err(RainyError::PayloadTooLarge {
                message: "response body exceeds the configured safety limit".to_string(),
                max_bytes: limit,
            });
        }
        body.extend_from_slice(&chunk);
    }
    Ok(body)
}

/// Parses a numeric `Retry-After` header and caps it to one day.
pub(crate) fn retry_after_seconds(headers: &reqwest::header::HeaderMap) -> Option<u64> {
    headers
        .get("retry-after")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<u64>().ok())
        .map(|value| value.min(MAX_RETRY_AFTER_SECONDS))
}

/// A simple rate limiter retained only for source compatibility.
#[deprecated(note = "Use the optional governor-based rate limiting in RainyClient")]
#[derive(Debug)]
pub struct RateLimiter {
    requests_per_minute: u32,
    last_request: std::time::Instant,
    request_count: u32,
}

#[allow(deprecated)]
impl RateLimiter {
    /// Creates a limiter.
    pub fn new(requests_per_minute: u32) -> Self {
        Self {
            requests_per_minute,
            last_request: std::time::Instant::now(),
            request_count: 0,
        }
    }

    /// Waits until the next request may be sent.
    pub async fn wait_if_needed(&mut self) -> Result<()> {
        let now = std::time::Instant::now();
        let elapsed = now.duration_since(self.last_request);
        if elapsed >= Duration::from_secs(60) {
            self.request_count = 0;
            self.last_request = now;
        }
        if self.request_count >= self.requests_per_minute {
            tokio::time::sleep(Duration::from_secs(60) - elapsed).await;
            self.request_count = 0;
            self.last_request = std::time::Instant::now();
        }
        self.request_count = self.request_count.saturating_add(1);
        Ok(())
    }
}
