use crate::{
    auth::AuthConfig,
    error::{ApiErrorResponse, RainyError, Result},
    models::*,
    retry::{retry_with_backoff, RetryConfig},
};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT},
    Client, Response,
};
use std::time::Instant;

#[cfg(feature = "rate-limiting")]
use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter,
};

/// Main client for interacting with the Rainy API
pub struct RainyClient {
    client: Client,
    auth_config: AuthConfig,
    retry_config: RetryConfig,

    #[cfg(feature = "rate-limiting")]
    rate_limiter: Option<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RainyClient {
    /// Create a new client with an API key
    pub fn with_api_key(api_key: impl Into<String>) -> Result<Self> {
        let auth_config = AuthConfig::new(api_key);
        Self::with_config(auth_config)
    }

    /// Create a new client with custom configuration
    pub fn with_config(auth_config: AuthConfig) -> Result<Self> {
        // Validate configuration
        auth_config.validate()?;

        // Build HTTP client
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", auth_config.api_key)).map_err(|e| {
                RainyError::Authentication {
                    code: "INVALID_API_KEY".to_string(),
                    message: format!("Invalid API key format: {}", e),
                    retryable: false,
                }
            })?,
        );
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&auth_config.user_agent).map_err(|e| RainyError::Network {
                message: format!("Invalid user agent: {}", e),
                retryable: false,
                source_error: None,
            })?,
        );

        let client = Client::builder()
            .timeout(auth_config.timeout())
            .default_headers(headers)
            .build()
            .map_err(|e| RainyError::Network {
                message: format!("Failed to create HTTP client: {}", e),
                retryable: false,
                source_error: Some(e.to_string()),
            })?;

        let retry_config = RetryConfig::new(auth_config.max_retries);

        #[cfg(feature = "rate-limiting")]
        let rate_limiter = Some(RateLimiter::direct(Quota::per_second(
            std::num::NonZeroU32::new(10).unwrap(),
        )));

        Ok(Self {
            client,
            auth_config,
            retry_config,
            #[cfg(feature = "rate-limiting")]
            rate_limiter,
        })
    }

    /// Set custom retry configuration
    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    /// Get available models and providers
    pub async fn get_available_models(&self) -> Result<AvailableModels> {
        let url = format!("{}/api/v1/models", self.auth_config.base_url);

        let operation = || async {
            let response = self.client.get(&url).send().await?;
            self.handle_response(response).await
        };

        if self.auth_config.enable_retry {
            retry_with_backoff(&self.retry_config, operation).await
        } else {
            operation().await
        }
    }

    /// Create a chat completion
    pub async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<(ChatCompletionResponse, RequestMetadata)> {
        #[cfg(feature = "rate-limiting")]
        if let Some(ref limiter) = self.rate_limiter {
            limiter.until_ready().await;
        }

        let url = format!("{}/api/v1/chat/completions", self.auth_config.base_url);
        let start_time = Instant::now();

        let operation = || async {
            let response = self.client.post(&url).json(&request).send().await?;

            let metadata = self.extract_metadata(&response, start_time);
            let chat_response: ChatCompletionResponse = self.handle_response(response).await?;

            Ok((chat_response, metadata))
        };

        if self.auth_config.enable_retry {
            retry_with_backoff(&self.retry_config, operation).await
        } else {
            operation().await
        }
    }

    /// Create a simple chat completion with just a prompt
    pub async fn simple_chat(
        &self,
        model: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Result<String> {
        let request = ChatCompletionRequest::new(model, vec![ChatMessage::user(prompt)]);

        let (response, _) = self.chat_completion(request).await?;

        Ok(response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .unwrap_or_default())
    }

    /// Handle HTTP response and extract typed data
    pub(crate) async fn handle_response<T>(&self, response: Response) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let headers = response.headers().clone();
        let request_id = headers
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        if status.is_success() {
            let text = response.text().await?;
            serde_json::from_str(&text).map_err(|e| RainyError::Serialization {
                message: format!("Failed to parse response: {}", e),
                source_error: Some(e.to_string()),
            })
        } else {
            let text = response.text().await.unwrap_or_default();

            // Try to parse structured error response
            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&text) {
                let error = error_response.error;
                self.map_api_error(error, status.as_u16(), request_id)
            } else {
                // Fallback to generic error
                Err(RainyError::Api {
                    code: status.canonical_reason().unwrap_or("UNKNOWN").to_string(),
                    message: if text.is_empty() {
                        format!("HTTP {}", status.as_u16())
                    } else {
                        text
                    },
                    status_code: status.as_u16(),
                    retryable: status.is_server_error(),
                    request_id,
                })
            }
        }
    }

    /// Extract metadata from response headers
    fn extract_metadata(&self, response: &Response, start_time: Instant) -> RequestMetadata {
        let headers = response.headers();

        RequestMetadata {
            response_time: Some(start_time.elapsed().as_millis() as u64),
            provider: headers
                .get("x-provider")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            tokens_used: headers
                .get("x-tokens-used")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            credits_used: headers
                .get("x-credits-used")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            credits_remaining: headers
                .get("x-credits-remaining")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            request_id: headers
                .get("x-request-id")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
        }
    }

    /// Map API error response to RainyError
    fn map_api_error<T>(
        &self,
        error: crate::error::ApiErrorDetails,
        status_code: u16,
        request_id: Option<String>,
    ) -> Result<T> {
        let retryable = error.retryable.unwrap_or(status_code >= 500);

        let rainy_error = match error.code.as_str() {
            "INVALID_API_KEY" | "EXPIRED_API_KEY" => RainyError::Authentication {
                code: error.code,
                message: error.message,
                retryable: false,
            },
            "INSUFFICIENT_CREDITS" => {
                // Extract credit info from details if available
                let (current_credits, estimated_cost, reset_date) =
                    if let Some(details) = error.details {
                        let current = details
                            .get("current_credits")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        let cost = details
                            .get("estimated_cost")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        let reset = details
                            .get("reset_date")
                            .and_then(|v| v.as_str())
                            .map(String::from);
                        (current, cost, reset)
                    } else {
                        (0.0, 0.0, None)
                    };

                RainyError::InsufficientCredits {
                    code: error.code,
                    message: error.message,
                    current_credits,
                    estimated_cost,
                    reset_date,
                }
            }
            "RATE_LIMIT_EXCEEDED" => {
                let retry_after = error
                    .details
                    .as_ref()
                    .and_then(|d| d.get("retry_after"))
                    .and_then(|v| v.as_u64());

                RainyError::RateLimit {
                    code: error.code,
                    message: error.message,
                    retry_after,
                    current_usage: None,
                }
            }
            "INVALID_REQUEST" | "MISSING_REQUIRED_FIELD" | "INVALID_MODEL" => {
                RainyError::InvalidRequest {
                    code: error.code,
                    message: error.message,
                    details: error.details,
                }
            }
            "PROVIDER_ERROR" | "PROVIDER_UNAVAILABLE" => {
                let provider = error
                    .details
                    .as_ref()
                    .and_then(|d| d.get("provider"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();

                RainyError::Provider {
                    code: error.code,
                    message: error.message,
                    provider,
                    retryable,
                }
            }
            _ => RainyError::Api {
                code: error.code,
                message: error.message,
                status_code,
                retryable,
                request_id: request_id.clone(),
            },
        };

        Err(rainy_error)
    }

    /// Get the current authentication configuration
    pub fn auth_config(&self) -> &AuthConfig {
        &self.auth_config
    }

    /// Get the base URL being used
    pub fn base_url(&self) -> &str {
        &self.auth_config.base_url
    }

    /// Get the HTTP client (for internal use by endpoints)
    pub(crate) fn http_client(&self) -> &Client {
        &self.client
    }

    // Legacy methods for backward compatibility

    /// Make a generic request (internal method, kept for endpoint compatibility)
    pub(crate) async fn make_request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<serde_json::Value>,
    ) -> Result<T> {
        #[cfg(feature = "rate-limiting")]
        if let Some(ref limiter) = self.rate_limiter {
            limiter.until_ready().await;
        }

        let url = format!("{}/api/v1{}", self.auth_config.base_url, endpoint);
        let headers = self.auth_config.build_headers()?;

        let mut request = self.client.request(method, &url).headers(headers);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = request.send().await?;
        self.handle_response(response).await
    }
}

impl std::fmt::Debug for RainyClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RainyClient")
            .field("base_url", &self.auth_config.base_url)
            .field("timeout", &self.auth_config.timeout_seconds)
            .field("max_retries", &self.retry_config.max_retries)
            .finish()
    }
}
