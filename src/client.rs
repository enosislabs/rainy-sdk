use crate::{
    auth::AuthConfig,
    error::{ApiErrorResponse, RainyError, Result},
    models::*,
    retry::{RetryConfig, retry_with_backoff},
};
use eventsource_stream::Eventsource;
use futures::{Stream, StreamExt};
use reqwest::{
    Client, Method, RequestBuilder, Response,
    header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT},
};
use secrecy::ExposeSecret;
use serde::Deserialize;
use std::future::Future;
use std::pin::Pin;
use std::time::Instant;

#[cfg(feature = "rate-limiting")]
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};

/// The main client for interacting with the Rainy API.
///
/// `RainyClient` provides a convenient and high-level interface for making requests
/// to the various endpoints of the Rainy API. It handles authentication, rate limiting,
/// and retries automatically.
///
/// # Examples
///
/// ```rust,no_run
/// use rainy_sdk::{RainyClient, Result};
///
/// #[tokio::main]
/// async fn main() -> Result<()> {
///     // Create a client using an API key from an environment variable
///     let api_key = std::env::var("RAINY_API_KEY").expect("RAINY_API_KEY not set");
///     let client = RainyClient::with_api_key(api_key)?;
///
///     // Use the client to make API calls
///     let models = client.get_available_models().await?;
///     println!("Available models: {:?}", models);
///
///     Ok(())
/// }
/// ```
pub struct RainyClient {
    /// The underlying `reqwest::Client` used for making HTTP requests.
    client: Client,
    /// The authentication configuration for the client.
    auth_config: AuthConfig,
    /// The retry configuration for handling failed requests.
    retry_config: RetryConfig,

    /// An optional rate limiter to control the request frequency.
    /// This is only available when the `rate-limiting` feature is enabled.
    #[cfg(feature = "rate-limiting")]
    rate_limiter: Option<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RainyClient {
    pub(crate) fn root_url(&self, path: &str) -> String {
        let normalized = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        format!(
            "{}{}",
            self.auth_config.base_url.trim_end_matches('/'),
            normalized
        )
    }

    pub(crate) fn api_v1_url(&self, path: &str) -> String {
        let normalized = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        match self.auth_config.api_base_url.as_deref() {
            Some(api_base_url) => {
                format!("{}{}", api_base_url.trim_end_matches('/'), normalized)
            }
            None => format!(
                "{}/api/v1{}",
                self.auth_config.base_url.trim_end_matches('/'),
                normalized
            ),
        }
    }

    /// Creates a new `RainyClient` with the given API key.
    ///
    /// This is the simplest way to create a client. It uses default settings for the base URL,
    /// timeout, and retries.
    ///
    /// # Arguments
    ///
    /// * `api_key` - Your Rainy API key.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `RainyClient` or a `RainyError` if initialization fails.
    pub fn with_api_key(api_key: impl Into<String>) -> Result<Self> {
        let auth_config = AuthConfig::new(api_key);
        Self::with_config(auth_config)
    }

    /// Creates a new `RainyClient` with a custom `AuthConfig`.
    ///
    /// This allows for more advanced configuration, such as setting a custom base URL or timeout.
    ///
    /// # Arguments
    ///
    /// * `auth_config` - The authentication configuration to use.
    ///
    /// # Returns
    ///
    /// A `Result` containing the new `RainyClient` or a `RainyError` if initialization fails.
    pub fn with_config(auth_config: AuthConfig) -> Result<Self> {
        // Validate configuration
        auth_config.validate()?;

        // Build HTTP client
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", auth_config.api_key.expose_secret()))
                .map_err(|e| RainyError::Authentication {
                    code: "INVALID_API_KEY".to_string(),
                    message: format!("Invalid API key format: {}", e),
                    retryable: false,
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
            .tls_backend_rustls()
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .https_only(true)
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

    /// Sets a custom retry configuration for the client.
    ///
    /// This allows you to override the default retry behavior.
    ///
    /// # Arguments
    ///
    /// * `retry_config` - The new retry configuration.
    ///
    /// # Returns
    ///
    /// The `RainyClient` instance with the updated retry configuration.
    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    pub(crate) async fn wait_for_slot(&self) {
        #[cfg(feature = "rate-limiting")]
        if let Some(ref limiter) = self.rate_limiter {
            limiter.until_ready().await;
        }
    }

    async fn execute_with_retry<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        self.wait_for_slot().await;

        if self.auth_config.enable_retry {
            retry_with_backoff(&self.retry_config, operation).await
        } else {
            operation().await
        }
    }

    pub(crate) fn api_request(&self, method: Method, endpoint: &str) -> RequestBuilder {
        self.client.request(method, self.api_v1_url(endpoint))
    }

    pub(crate) fn root_request(&self, method: Method, endpoint: &str) -> RequestBuilder {
        self.client.request(method, self.root_url(endpoint))
    }

    pub(crate) async fn send_request(&self, request: RequestBuilder) -> Result<Response> {
        request.send().await.map_err(|e| RainyError::Network {
            message: format!("Failed to send request: {e}"),
            retryable: true,
            source_error: Some(e.to_string()),
        })
    }

    /// Retrieves the list of available models and providers from the API.
    ///
    /// # Returns
    ///
    /// A `Result` containing an `AvailableModels` struct on success, or a `RainyError` on failure.
    pub async fn get_available_models(&self) -> Result<AvailableModels> {
        #[derive(Deserialize)]
        struct ModelListItem {
            id: String,
        }
        #[derive(Deserialize)]
        struct ModelsData {
            data: Vec<ModelListItem>,
        }
        #[derive(Deserialize)]
        struct Envelope {
            data: ModelsData,
        }

        let url = self.api_v1_url("/models");

        let operation = || async {
            let response = self.send_request(self.client.get(&url)).await?;
            let envelope: Envelope = self.handle_response(response).await?;

            let mut providers = std::collections::HashMap::<String, Vec<String>>::new();
            for item in envelope.data.data {
                let provider = item
                    .id
                    .split_once('/')
                    .map(|(p, _)| p.to_string())
                    .unwrap_or_else(|| "rainy".to_string());
                providers.entry(provider).or_default().push(item.id);
            }

            let total_models = providers.values().map(std::vec::Vec::len).sum();
            let mut active_providers = providers.keys().cloned().collect::<Vec<_>>();
            active_providers.sort();

            Ok(AvailableModels {
                providers,
                total_models,
                active_providers,
            })
        };

        self.execute_with_retry(operation).await
    }

    /// Creates a chat completion based on the provided request.
    ///
    /// # Arguments
    ///
    /// * `request` - A `ChatCompletionRequest` containing the model, messages, and other parameters.
    ///
    /// # Returns
    ///
    /// A `Result` containing a tuple of `(ChatCompletionResponse, RequestMetadata)` on success,
    /// or a `RainyError` on failure.
    pub async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<(ChatCompletionResponse, RequestMetadata)> {
        let url = self.api_v1_url("/chat/completions");
        let start_time = Instant::now();

        let operation = || async {
            let response = self
                .send_request(self.client.post(&url).json(&request))
                .await?;

            let metadata = self.extract_metadata(&response, start_time);
            let chat_response: ChatCompletionResponse = self.handle_response(response).await?;

            Ok((chat_response, metadata))
        };

        self.execute_with_retry(operation).await
    }

    /// Creates a chat completion in envelope mode (`X-Rainy-Response-Mode: envelope`).
    pub async fn chat_completion_envelope(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<(RainyEnvelope<ChatCompletionResponse>, RequestMetadata)> {
        let url = self.api_v1_url("/chat/completions");
        let start_time = Instant::now();

        let operation = || async {
            let response = self
                .send_request(
                    self.client
                        .post(&url)
                        .header("X-Rainy-Response-Mode", "envelope")
                        .json(&request),
                )
                .await?;

            let metadata = self.extract_metadata(&response, start_time);
            let chat_response: RainyEnvelope<ChatCompletionResponse> =
                self.handle_response(response).await?;
            Ok((chat_response, metadata))
        };

        self.execute_with_retry(operation).await
    }

    /// Creates an OpenAI-compatible chat completion in envelope mode.
    pub async fn openai_chat_completion_envelope(
        &self,
        request: OpenAIChatCompletionRequest,
    ) -> Result<(RainyEnvelope<OpenAIChatCompletionResponse>, RequestMetadata)> {
        let url = self.api_v1_url("/chat/completions");
        let start_time = Instant::now();

        let operation = || async {
            let response = self
                .send_request(
                    self.client
                        .post(&url)
                        .header("X-Rainy-Response-Mode", "envelope")
                        .json(&request),
                )
                .await?;

            let metadata = self.extract_metadata(&response, start_time);
            let chat_response: RainyEnvelope<OpenAIChatCompletionResponse> =
                self.handle_response(response).await?;
            Ok((chat_response, metadata))
        };

        self.execute_with_retry(operation).await
    }

    /// Creates a streaming chat completion based on the provided request.
    ///
    /// # Arguments
    ///
    /// * `request` - A `ChatCompletionRequest` containing the model, messages, and other parameters.
    ///
    /// # Returns
    ///
    /// A `Result` containing a stream of OpenAI-style `chat.completion.chunk` events on success,
    /// or a `RainyError` on failure.
    pub async fn chat_completion_stream(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionStreamResponse>> + Send>>> {
        // Ensure stream is set to true
        request.stream = Some(true);

        let url = self.api_v1_url("/chat/completions");

        // Note: Retries are more complex with streams, so we only retry the initial connection
        let operation = || async {
            let response = self
                .client
                .post(&url)
                .json(&request)
                .send()
                .await
                .map_err(|e| RainyError::Network {
                    message: format!("Failed to send request: {}", e),
                    retryable: true,
                    source_error: Some(e.to_string()),
                })?;

            let events = self.handle_chat_stream_response(response).await?;
            let stream = events.filter_map(|event| async move {
                match event {
                    Ok(ChatStreamEvent::Chunk(chunk)) => Some(Ok(chunk)),
                    Ok(ChatStreamEvent::Billing(_)) | Ok(ChatStreamEvent::Raw(_)) => None,
                    Err(error) => Some(Err(error)),
                }
            });

            Ok(Box::pin(stream)
                as Pin<
                    Box<dyn Stream<Item = Result<ChatCompletionStreamResponse>> + Send>,
                >)
        };

        self.execute_with_retry(operation).await
    }

    /// Creates a Responses API completion (`POST /api/v1/responses`) in raw mode.
    pub async fn create_response(
        &self,
        request: ResponsesRequest,
    ) -> Result<(ResponsesApiResponse, RequestMetadata)> {
        let url = self.api_v1_url("/responses");
        let start_time = Instant::now();

        let operation = || async {
            let response = self
                .send_request(self.client.post(&url).json(&request))
                .await?;
            let metadata = self.extract_metadata(&response, start_time);
            let api_response: ResponsesApiResponse = self.handle_response(response).await?;
            Ok((api_response, metadata))
        };

        self.execute_with_retry(operation).await
    }

    /// Creates a Responses API completion in envelope mode (`X-Rainy-Response-Mode: envelope`).
    pub async fn create_response_envelope(
        &self,
        request: ResponsesRequest,
    ) -> Result<(RainyEnvelope<ResponsesApiResponse>, RequestMetadata)> {
        let url = self.api_v1_url("/responses");
        let start_time = Instant::now();

        let operation = || async {
            let response = self
                .send_request(
                    self.client
                        .post(&url)
                        .header("X-Rainy-Response-Mode", "envelope")
                        .json(&request),
                )
                .await?;
            let metadata = self.extract_metadata(&response, start_time);
            let api_response: RainyEnvelope<ResponsesApiResponse> =
                self.handle_response(response).await?;
            Ok((api_response, metadata))
        };

        self.execute_with_retry(operation).await
    }

    /// Creates a streaming Responses API completion and returns SSE events.
    pub async fn create_response_stream(
        &self,
        mut request: ResponsesRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ResponsesStreamEvent>> + Send>>> {
        request.stream = Some(true);

        let url = self.api_v1_url("/responses");

        let operation = || async {
            let response = self
                .send_request(self.client.post(&url).json(&request))
                .await?;

            self.handle_stream_response(response).await
        };

        self.execute_with_retry(operation).await
    }

    /// Creates a chat completion stream returning typed events (OpenAI chunks + Rainy native events).
    pub async fn chat_completion_stream_events(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatStreamEvent>> + Send>>> {
        request.stream = Some(true);

        let url = self.api_v1_url("/chat/completions");
        let operation = || async {
            let response = self
                .send_request(self.client.post(&url).json(&request))
                .await?;

            self.handle_chat_stream_response(response).await
        };

        self.execute_with_retry(operation).await
    }

    /// Retrieves `/api/v1/models/catalog` entries including `rainy_capabilities` metadata.
    pub async fn get_models_catalog(&self) -> Result<Vec<ModelCatalogItem>> {
        #[derive(Deserialize)]
        struct ModelsCatalogData {
            data: Vec<ModelCatalogItem>,
        }
        #[derive(Deserialize)]
        struct Envelope {
            data: ModelsCatalogData,
        }

        let url = self.api_v1_url("/models/catalog");
        let operation = || async {
            let response = self.send_request(self.client.get(&url)).await?;
            let envelope: Envelope = self.handle_response(response).await?;
            Ok(envelope.data.data)
        };

        self.execute_with_retry(operation).await
    }

    /// Retrieves catalog and filters/sorts models using SDK selector criteria.
    pub async fn select_models(
        &self,
        criteria: ModelSelectionCriteria,
    ) -> Result<Vec<ModelCatalogItem>> {
        let catalog = self.get_models_catalog().await?;
        Ok(crate::models::select_models(&catalog, &criteria))
    }

    /// Builds provider-aware reasoning payload from a catalog entry and preference.
    pub fn build_reasoning_config(
        &self,
        model: &ModelCatalogItem,
        preference: &ReasoningPreference,
    ) -> Option<serde_json::Value> {
        crate::models::build_reasoning_config(model, preference)
    }

    /// Creates a simple chat completion with a single user prompt.
    ///
    /// This is a convenience method for simple use cases where you only need to send a single
    /// prompt to a model and get a text response.
    ///
    /// # Arguments
    ///
    /// * `model` - The name of the model to use for the completion.
    /// * `prompt` - The user's prompt.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `String` response from the model, or a `RainyError` on failure.
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

    /// Handles the HTTP response, deserializing the body into a given type `T` on success,
    /// or mapping the error to a `RainyError` on failure.
    ///
    /// This is an internal method used by the various endpoint functions.
    pub(crate) async fn handle_response<T>(&self, response: Response) -> Result<T>
    where
        T: serde::de::DeserializeOwned,
    {
        let status = response.status();
        let request_id = response
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        if status.is_success() {
            let body = response.bytes().await?;
            serde_json::from_slice(&body).map_err(|e| RainyError::Serialization {
                message: format!("Failed to parse response: {}", e),
                source_error: Some(e.to_string()),
            })
        } else {
            let text = response.text().await.unwrap_or_default();
            self.handle_error_text(status, request_id, text)
        }
    }

    /// Handles the HTTP response for streaming requests.
    pub(crate) async fn handle_stream_response<T>(
        &self,
        response: Response,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<T>> + Send>>>
    where
        T: serde::de::DeserializeOwned + Send + 'static,
    {
        let status = response.status();
        let request_id = response
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return self.handle_error_text(status, request_id, text);
        }

        let stream = response
            .bytes_stream()
            .eventsource()
            .filter_map(|event| async move {
                match event {
                    Ok(event) => {
                        let payload = event.data.trim();
                        if payload.is_empty() || payload.eq_ignore_ascii_case("[DONE]") {
                            return None;
                        }

                        match serde_json::from_str::<T>(payload) {
                            Ok(chunk) => Some(Ok(chunk)),
                            Err(e) => Some(Err(RainyError::Serialization {
                                message: format!("Failed to parse stream chunk: {}", e),
                                source_error: Some(e.to_string()),
                            })),
                        }
                    }
                    Err(e) => Some(Err(RainyError::Network {
                        message: format!("Stream error: {}", e),
                        retryable: true,
                        source_error: Some(e.to_string()),
                    })),
                }
            });

        Ok(Box::pin(stream))
    }

    pub(crate) async fn handle_chat_stream_response(
        &self,
        response: Response,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatStreamEvent>> + Send>>> {
        let status = response.status();
        let request_id = response
            .headers()
            .get("x-request-id")
            .and_then(|v| v.to_str().ok())
            .map(String::from);

        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            return self.handle_error_text(status, request_id, text);
        }

        let stream = response
            .bytes_stream()
            .eventsource()
            .filter_map(|event| async move {
                match event {
                    Ok(event) => {
                        let payload = event.data.trim();
                        if payload.is_empty() || payload.eq_ignore_ascii_case("[DONE]") {
                            return None;
                        }

                        let event_name = event.event.trim();
                        let event_name = (!event_name.is_empty()).then_some(event_name);

                        match serde_json::from_str::<serde_json::Value>(payload) {
                            Ok(value) => {
                                Some(Ok(ChatStreamEvent::from_sse_event(event_name, value)))
                            }
                            Err(e) => Some(Err(RainyError::Serialization {
                                message: format!("Failed to parse stream chunk: {}", e),
                                source_error: Some(e.to_string()),
                            })),
                        }
                    }
                    Err(e) => Some(Err(RainyError::Network {
                        message: format!("Stream error: {}", e),
                        retryable: true,
                        source_error: Some(e.to_string()),
                    })),
                }
            });

        Ok(Box::pin(stream))
    }

    fn handle_error_text<T>(
        &self,
        status: reqwest::StatusCode,
        request_id: Option<String>,
        text: String,
    ) -> Result<T> {
        if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&text) {
            let error = error_response.error;
            self.map_api_error(error, status.as_u16(), request_id)
        } else {
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

    /// Extracts request metadata from the HTTP response headers.
    ///
    /// This is an internal method.
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
            compat_warnings: headers
                .get("x-rainy-compat-warnings")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            response_mode: headers
                .get("x-rainy-response-mode")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            billing_plan: headers
                .get("x-rainy-billing-plan")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            rainy_credits_charged: headers
                .get("x-rainy-credits-charged")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
            rainy_daily_credits_remaining: headers
                .get("x-rainy-daily-credits-remaining")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            rainy_sanitized_params: headers
                .get("x-rainy-sanitized-params")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            rainy_billing_adjustment: headers
                .get("x-rainy-billing-adjustment")
                .and_then(|v| v.to_str().ok())
                .map(String::from),
            rainy_billing_outstanding_credits: headers
                .get("x-rainy-billing-outstanding-credits")
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.parse().ok()),
        }
    }

    /// Maps a structured API error response to a `RainyError`.
    ///
    /// This is an internal method.
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
            "MODEL_TIER_NOT_ALLOWED"
            | "MODEL_NOT_ALLOWED"
            | "MODEL_DISABLED_FOR_ORGANIZATION"
            | "MODEL_PRIVACY_POLICY_INCOMPATIBLE"
            | "TOOLS_NOT_ALLOWED"
            | "REASONING_NOT_ALLOWED" => RainyError::AccessDenied {
                code: error.code,
                message: error.message,
                details: error.details,
            },
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

    /// Returns a reference to the current authentication configuration.
    pub fn auth_config(&self) -> &AuthConfig {
        &self.auth_config
    }

    /// Returns the base URL being used by the client.
    pub fn base_url(&self) -> &str {
        &self.auth_config.base_url
    }

    /// Returns the effective base URL used for versioned API endpoints.
    pub fn api_base_url(&self) -> String {
        self.auth_config.api_base_url.clone().unwrap_or_else(|| {
            format!("{}/api/v1", self.auth_config.base_url.trim_end_matches('/'))
        })
    }

    /// Retrieves the list of available models from the API.
    ///
    /// This method returns information about all models that are currently available
    /// through the Rainy API, including their compatibility status and supported parameters.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `AvailableModels` struct with model information.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::RainyClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("your-api-key")?;
    /// let models = client.list_available_models().await?;
    ///
    /// println!("Total models: {}", models.total_models);
    /// for (provider, model_list) in &models.providers {
    ///     println!("Provider {}: {:?}", provider, model_list);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_available_models(&self) -> Result<AvailableModels> {
        self.get_available_models().await
    }

    // Legacy methods for backward compatibility

    /// Makes a generic HTTP request to the API.
    ///
    /// This is an internal method kept for compatibility with endpoint implementations.
    pub(crate) async fn make_request<T: serde::de::DeserializeOwned>(
        &self,
        method: reqwest::Method,
        endpoint: &str,
        body: Option<serde_json::Value>,
    ) -> Result<T> {
        self.wait_for_slot().await;
        let mut request = self.api_request(method, endpoint);

        if let Some(body) = body {
            request = request.json(&body);
        }

        let response = self.send_request(request).await?;
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

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_api_key() -> String {
        format!("ra-{}", "a".repeat(48))
    }

    #[test]
    fn versioned_routes_use_the_rainy_prefix_by_default() {
        let client = RainyClient::with_config(
            AuthConfig::new(valid_api_key()).with_base_url("https://gateway.example.com/"),
        )
        .expect("build client");

        assert_eq!(
            client.api_v1_url("responses"),
            "https://gateway.example.com/api/v1/responses"
        );
    }

    #[test]
    fn versioned_routes_use_the_configured_api_base_url() {
        let client = RainyClient::with_config(
            AuthConfig::new(valid_api_key())
                .with_base_url("https://gateway.example.com")
                .with_api_base_url("https://responses.example.com/openai/v1/"),
        )
        .expect("build client");

        assert_eq!(
            client.api_v1_url("/responses"),
            "https://responses.example.com/openai/v1/responses"
        );
        assert_eq!(
            client.root_url("/health"),
            "https://gateway.example.com/health"
        );
    }
}
