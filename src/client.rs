use crate::auth::{AuthConfig, RateLimiter};
use crate::error::{RainyError, Result};
use reqwest::{Client as HttpClient, Response};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct RainyClient {
    pub(crate) http_client: HttpClient,
    pub(crate) config: AuthConfig,
    pub(crate) rate_limiter: Arc<Mutex<Option<RateLimiter>>>,
}

impl RainyClient {
    /// Creates a new RainyClient with the provided configuration.
    pub fn new(config: AuthConfig) -> Result<Self> {
        config.validate()?;

        let http_client = HttpClient::builder().timeout(config.timeout).build()?;

        Ok(Self {
            http_client,
            config,
            rate_limiter: Arc::new(Mutex::new(None)),
        })
    }

    /// Creates a new RainyClient with just an API key.
    ///
    /// This is a convenience method that automatically configures the client
    /// to connect to `https://api.enosislabs.com`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use rainy_sdk::RainyClient;
    ///
    /// let client = RainyClient::with_api_key("your-api-key")?;
    /// # Ok::<(), rainy_sdk::error::RainyError>(())
    /// ```
    pub fn with_api_key<S: Into<String>>(api_key: S) -> Result<Self> {
        Self::new(AuthConfig::new().with_api_key(api_key))
    }

    pub fn with_rate_limit(mut self, requests_per_minute: u32) -> Self {
        let rate_limiter = RateLimiter::new(requests_per_minute);
        self.rate_limiter = Arc::new(Mutex::new(Some(rate_limiter)));
        self
    }

    pub(crate) async fn make_request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<serde_json::Value>,
    ) -> Result<T> {
        // Apply rate limiting if configured
        if let Some(rate_limiter) = self.rate_limiter.lock().await.as_mut() {
            rate_limiter.wait_if_needed().await?;
        }

        let url = format!("{}/api/v1{}", self.config.base_url, endpoint);
        let headers = self.config.build_headers()?;

        let mut request = self.http_client.request(method, &url).headers(headers);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }

    pub(crate) async fn handle_response<T: serde::de::DeserializeOwned>(
        &self,
        response: Response,
    ) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            let result = response.json::<T>().await?;
            Ok(result)
        } else {
            let request_id = response.headers()
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .map(String::from);
            
            let error_text = response.text().await.unwrap_or_default();

            match status {
                reqwest::StatusCode::UNAUTHORIZED => Err(RainyError::Authentication {
                    code: "UNAUTHORIZED".to_string(),
                    message: "Invalid API key".to_string(),
                    retryable: false,
                }),
                reqwest::StatusCode::FORBIDDEN => Err(RainyError::Authentication {
                    code: "FORBIDDEN".to_string(),
                    message: "Insufficient permissions".to_string(),
                    retryable: false,
                }),
                reqwest::StatusCode::TOO_MANY_REQUESTS => Err(RainyError::RateLimit {
                    code: "RATE_LIMIT_EXCEEDED".to_string(),
                    message: "Rate limit exceeded".to_string(),
                    retry_after: None,
                    current_usage: None,
                }),
                reqwest::StatusCode::BAD_REQUEST => {
                    // Try to parse as API error
                    if let Ok(api_error) = serde_json::from_str::<crate::error::ApiErrorResponse>(&error_text) {
                        Err(RainyError::InvalidRequest {
                            code: api_error.error.code,
                            message: api_error.error.message,
                            details: api_error.error.details,
                        })
                    } else {
                        Err(RainyError::InvalidRequest {
                            code: "BAD_REQUEST".to_string(),
                            message: error_text,
                            details: None,
                        })
                    }
                }
                _ => Err(RainyError::Api {
                    code: status.canonical_reason().unwrap_or("UNKNOWN").to_string(),
                    message: error_text,
                    status_code: status.as_u16(),
                    retryable: status.is_server_error(),
                    request_id,
                }),
            }
        }
    }

    // Health check methods will be moved to a separate file in Phase 3.
    // User account management methods will be moved to a separate file in Phase 3.
    // API key management methods will be moved to a separate file in Phase 3.
    // Usage and credits methods will be moved to a separate file in Phase 3.
    // Chat completions methods will be moved to a separate file in Phase 3.
    // Admin functions will be moved to a separate file in Phase 3.
}
