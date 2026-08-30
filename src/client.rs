//! Shared HTTP transport and Rainy/OpenAI-compatible protocol operations.

use crate::{
    auth::{
        AuthConfig, AuthScheme, GENERAL_REQUEST_BODY_BYTES, MAX_ERROR_BODY_BYTES,
        MAX_RESPONSE_BODY_BYTES, MODEL_REQUEST_BODY_BYTES, read_limited_response_body,
        retry_after_seconds, safe_url_label, serialize_json_body,
    },
    error::{ApiErrorDetails, RainyError, Result},
    models::*,
    retry::{RetryConfig, retry_with_backoff},
    sse::parse_sse_stream,
};
use futures::{Stream, StreamExt};
use reqwest::{
    Client, Method, RequestBuilder, Response,
    header::{AUTHORIZATION, HeaderMap, HeaderValue, USER_AGENT},
};
use secrecy::ExposeSecret;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::{future::Future, pin::Pin, time::Instant};

#[cfg(feature = "rate-limiting")]
use governor::{
    Quota, RateLimiter,
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
};

const DEFAULT_ANTHROPIC_VERSION: &str = "2023-06-01";

fn normalize_path(path: &str) -> String {
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

fn encode_path_segment(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(char::from(b"0123456789ABCDEF"[(byte >> 4) as usize]));
            encoded.push(char::from(b"0123456789ABCDEF"[(byte & 0x0f) as usize]));
        }
    }
    encoded
}

fn request_body_limit(endpoint: &str) -> usize {
    match endpoint {
        "/chat/completions" | "/responses" | "/messages" | "/embeddings" => {
            MODEL_REQUEST_BODY_BYTES
        }
        _ => GENERAL_REQUEST_BODY_BYTES,
    }
}

/// Main reusable API-key client.
///
/// The client owns one configured `reqwest::Client` and exposes protocol
/// operations over that transport. It does not perform model discovery before
/// inference and it never retries an ordinary inference POST.
pub struct RainyClient {
    client: Client,
    auth_config: AuthConfig,
    retry_config: RetryConfig,
    #[cfg(feature = "rate-limiting")]
    rate_limiter: Option<RateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
}

impl RainyClient {
    pub(crate) fn root_url(&self, path: &str) -> String {
        format!(
            "{}{}",
            self.auth_config.base_url.trim_end_matches('/'),
            normalize_path(path)
        )
    }

    pub(crate) fn api_v1_url(&self, path: &str) -> String {
        let normalized = normalize_path(path);
        if let Some(api_base_url) = self.auth_config.api_base_url.as_deref() {
            return format!("{}{}", api_base_url.trim_end_matches('/'), normalized);
        }

        let base = self.auth_config.base_url.trim_end_matches('/');
        let use_configured_path = url::Url::parse(base)
            .ok()
            .is_some_and(|url| url.path() != "" && url.path() != "/");
        if use_configured_path {
            format!("{base}{normalized}")
        } else {
            format!("{base}/api/v1{normalized}")
        }
    }

    /// Creates a client using a protocol-neutral API key.
    pub fn with_api_key(api_key: impl Into<String>) -> Result<Self> {
        Self::with_config(AuthConfig::new(api_key))
    }

    /// Creates a client from explicit configuration.
    pub fn with_config(auth_config: AuthConfig) -> Result<Self> {
        auth_config.validate()?;

        // Only non-sensitive defaults are installed globally. Authentication
        // is attached per protocol so a Messages request can never inherit a
        // Bearer header by accident.
        let mut headers = HeaderMap::new();
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&auth_config.user_agent).map_err(|_| {
                RainyError::InvalidRequest {
                    code: "INVALID_USER_AGENT".to_string(),
                    message: "User-Agent contains invalid header characters".to_string(),
                    details: None,
                }
            })?,
        );

        let client = Client::builder()
            .tls_backend_rustls()
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            // Authenticated redirects can disclose credentials to another
            // origin, so callers must configure the final service URL.
            .redirect(reqwest::redirect::Policy::none())
            .timeout(auth_config.timeout())
            .default_headers(headers)
            .build()
            .map_err(|_| RainyError::Network {
                message: "Failed to create HTTP client".to_string(),
                retryable: false,
                source_error: None,
            })?;

        #[cfg(feature = "rate-limiting")]
        let rate_limiter = Some(RateLimiter::direct(Quota::per_second(
            std::num::NonZeroU32::new(10).expect("non-zero quota"),
        )));

        Ok(Self {
            client,
            retry_config: RetryConfig::new(auth_config.max_retries),
            auth_config,
            #[cfg(feature = "rate-limiting")]
            rate_limiter,
        })
    }

    /// Replaces the retry configuration used for safe operations.
    pub fn with_retry_config(mut self, retry_config: RetryConfig) -> Self {
        self.retry_config = retry_config;
        self
    }

    pub(crate) async fn wait_for_slot(&self) {
        #[cfg(feature = "rate-limiting")]
        if let Some(limiter) = &self.rate_limiter {
            limiter.until_ready().await;
        }
    }

    async fn execute_safe<F, Fut, T>(&self, operation: F) -> Result<T>
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

    async fn execute_once<F, Fut, T>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T>>,
    {
        self.wait_for_slot().await;
        operation().await
    }

    /// Builds an OpenAI-compatible Bearer-authenticated request.
    pub(crate) fn api_request(&self, method: Method, endpoint: &str) -> RequestBuilder {
        let value = format!("Bearer {}", self.auth_config.api_key.expose_secret());
        self.client
            .request(method, self.api_v1_url(endpoint))
            .header(AUTHORIZATION, value)
    }

    /// Builds an unauthenticated root-level request, used by health checks.
    pub(crate) fn root_request(&self, method: Method, endpoint: &str) -> RequestBuilder {
        self.client.request(method, self.root_url(endpoint))
    }

    /// Builds a native Anthropic request without a Bearer header.
    pub(crate) fn anthropic_request(
        &self,
        method: Method,
        endpoint: &str,
        version: &str,
    ) -> Result<RequestBuilder> {
        let headers = self
            .auth_config
            .build_protocol_headers(AuthScheme::XApiKey, Some(version))?;
        Ok(self
            .client
            .request(method, self.api_v1_url(endpoint))
            .headers(headers))
    }

    /// Serializes and bounds an OpenAI-compatible request body.
    pub(crate) fn json_request<T: serde::Serialize>(
        &self,
        method: Method,
        endpoint: &str,
        body: &T,
    ) -> Result<RequestBuilder> {
        let body = serialize_json_body(body, request_body_limit(endpoint))?;
        Ok(self
            .api_request(method, endpoint)
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body))
    }

    /// Serializes and bounds a native Messages request body.
    pub(crate) fn anthropic_json_request<T: serde::Serialize>(
        &self,
        method: Method,
        endpoint: &str,
        body: &T,
    ) -> Result<RequestBuilder> {
        let body = serialize_json_body(body, request_body_limit(endpoint))?;
        Ok(self
            .anthropic_request(method, endpoint, DEFAULT_ANTHROPIC_VERSION)?
            .header(reqwest::header::CONTENT_TYPE, "application/json")
            .body(body))
    }

    pub(crate) async fn send_request(&self, request: RequestBuilder) -> Result<Response> {
        request.send().await.map_err(|error| {
            if error.is_timeout() {
                RainyError::Timeout {
                    message: "Request timed out".to_string(),
                    duration_ms: self.auth_config.timeout_seconds.saturating_mul(1000),
                }
            } else {
                RainyError::Network {
                    message: if error.is_connect() {
                        "Could not connect to the service".to_string()
                    } else {
                        "The HTTP request failed".to_string()
                    },
                    retryable: error.is_connect() || error.is_request(),
                    source_error: None,
                }
            }
        })
    }

    /// Performs a safe explicit model-list request.
    pub async fn list_models(&self) -> Result<ModelList> {
        self.execute_safe(|| async {
            let response = self
                .send_request(self.api_request(Method::GET, "/models"))
                .await?;
            let value: Value = self.handle_response(response).await?;
            decode_model_list(value)
        })
        .await
    }

    /// Returns a compatibility summary of the public model list.
    pub async fn get_available_models(&self) -> Result<AvailableModels> {
        let models = self.list_models().await?;
        let mut providers = std::collections::HashMap::<String, Vec<String>>::new();
        for item in models.data {
            let owner = item
                .id
                .split_once('/')
                .map(|(owner, _)| owner.to_string())
                .unwrap_or_else(|| "default".to_string());
            providers.entry(owner).or_default().push(item.id);
        }
        let total_models = providers.values().map(Vec::len).sum();
        let mut active_providers = providers.keys().cloned().collect::<Vec<_>>();
        active_providers.sort();
        Ok(AvailableModels {
            providers,
            total_models,
            active_providers,
        })
    }

    /// Alias for [`Self::get_available_models`].
    pub async fn list_available_models(&self) -> Result<AvailableModels> {
        self.get_available_models().await
    }

    /// Retrieves one public model record.
    pub async fn get_model(&self, model_id: &str) -> Result<ModelListItem> {
        let endpoint = format!("/models/{}", encode_path_segment(model_id));
        self.execute_safe(|| async {
            let response = self
                .send_request(self.api_request(Method::GET, &endpoint))
                .await?;
            let value: Value = self.handle_response(response).await?;
            let data = value.get("data").cloned().unwrap_or(value);
            serde_json::from_value(data).map_err(|error| RainyError::Serialization {
                message: "failed to decode model response".to_string(),
                source_error: Some(error.to_string()),
            })
        })
        .await
    }

    /// Retrieves the public model catalog extension.
    pub async fn get_models_catalog(&self) -> Result<Vec<ModelCatalogItem>> {
        self.execute_safe(|| async {
            let response = self
                .send_request(self.api_request(Method::GET, "/models/catalog"))
                .await?;
            let value: Value = self.handle_response(response).await?;
            decode_catalog(value)
        })
        .await
    }

    /// Fetches the public catalog and applies local selection criteria.
    pub async fn select_models(
        &self,
        criteria: ModelSelectionCriteria,
    ) -> Result<Vec<ModelCatalogItem>> {
        Ok(select_models(&self.get_models_catalog().await?, &criteria))
    }

    /// Builds a literal reasoning object using public catalog declarations.
    pub fn build_reasoning_config(
        &self,
        model: &ModelCatalogItem,
        preference: &ReasoningPreference,
    ) -> Option<Value> {
        build_reasoning_config(model, preference)
    }

    /// Creates a compact text Chat completion.
    pub async fn chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<(ChatCompletionResponse, RequestMetadata)> {
        request
            .validate_openai_compatibility()
            .map_err(RainyError::ValidationError)?;
        let started = Instant::now();
        let response = self
            .execute_once(|| async {
                self.send_request(self.json_request(Method::POST, "/chat/completions", &request)?)
                    .await
            })
            .await?;
        let metadata = self.extract_metadata(&response, started);
        let result = self.handle_response(response).await?;
        Ok((result, metadata))
    }

    /// Creates a compact Chat completion without metadata.
    pub async fn create_chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        Ok(self.chat_completion(request).await?.0)
    }

    /// Creates a compact Chat completion in Rainy envelope mode.
    pub async fn chat_completion_envelope(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<(RainyEnvelope<ChatCompletionResponse>, RequestMetadata)> {
        request
            .validate_openai_compatibility()
            .map_err(RainyError::ValidationError)?;
        let started = Instant::now();
        let response = self
            .execute_once(|| async {
                self.send_request(
                    self.json_request(Method::POST, "/chat/completions", &request)?
                        .header("X-Rainy-Response-Mode", "envelope"),
                )
                .await
            })
            .await?;
        let metadata = self.extract_metadata(&response, started);
        Ok((self.handle_response(response).await?, metadata))
    }

    /// Creates a compact Chat completion in envelope mode without metadata.
    pub async fn create_chat_completion_envelope(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<RainyEnvelope<ChatCompletionResponse>> {
        Ok(self.chat_completion_envelope(request).await?.0)
    }

    /// Creates a compact streaming Chat completion.
    pub async fn chat_completion_stream(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionStreamResponse>> + Send>>> {
        request
            .validate_openai_compatibility()
            .map_err(RainyError::ValidationError)?;
        request.stream = Some(true);
        let events = self
            .execute_once(|| async {
                let response = self
                    .send_request(self.json_request(Method::POST, "/chat/completions", &request)?)
                    .await?;
                let events = self.handle_chat_stream_response(response).await?;
                let chunks = events.filter_map(|event| async move {
                    match event {
                        Ok(ChatStreamEvent::Chunk(chunk)) => Some(Ok(chunk)),
                        Ok(ChatStreamEvent::Billing(_))
                        | Ok(ChatStreamEvent::Unknown { .. })
                        | Ok(ChatStreamEvent::Raw(_)) => None,
                        Err(error) => Some(Err(error)),
                    }
                });
                Ok(Box::pin(chunks)
                    as Pin<
                        Box<dyn Stream<Item = Result<ChatCompletionStreamResponse>> + Send>,
                    >)
            })
            .await?;
        Ok(events)
    }

    /// Creates a typed streaming Chat completion.
    pub async fn chat_completion_stream_events(
        &self,
        mut request: ChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatStreamEvent>> + Send>>> {
        request
            .validate_openai_compatibility()
            .map_err(RainyError::ValidationError)?;
        request.stream = Some(true);
        self.execute_once(|| async {
            let response = self
                .send_request(self.json_request(Method::POST, "/chat/completions", &request)?)
                .await?;
            self.handle_chat_stream_response(response).await
        })
        .await
    }

    /// Creates an OpenAI-compatible Responses request.
    pub async fn create_response(
        &self,
        request: ResponsesRequest,
    ) -> Result<(ResponsesApiResponse, RequestMetadata)> {
        request.validate().map_err(RainyError::ValidationError)?;
        let started = Instant::now();
        let response = self
            .execute_once(|| async {
                self.send_request(self.json_request(Method::POST, "/responses", &request)?)
                    .await
            })
            .await?;
        let metadata = self.extract_metadata(&response, started);
        Ok((self.handle_response(response).await?, metadata))
    }

    /// Creates a Responses request without metadata.
    pub async fn response(&self, request: ResponsesRequest) -> Result<ResponsesApiResponse> {
        Ok(self.create_response(request).await?.0)
    }

    /// Creates a Responses request in Rainy envelope mode.
    pub async fn create_response_envelope(
        &self,
        request: ResponsesRequest,
    ) -> Result<(RainyEnvelope<ResponsesApiResponse>, RequestMetadata)> {
        request.validate().map_err(RainyError::ValidationError)?;
        let started = Instant::now();
        let response = self
            .execute_once(|| async {
                self.send_request(
                    self.json_request(Method::POST, "/responses", &request)?
                        .header("X-Rainy-Response-Mode", "envelope"),
                )
                .await
            })
            .await?;
        let metadata = self.extract_metadata(&response, started);
        Ok((self.handle_response(response).await?, metadata))
    }

    /// Creates a Responses stream with native event names preserved.
    pub async fn create_response_stream(
        &self,
        mut request: ResponsesRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ResponsesStreamEvent>> + Send>>> {
        request.validate().map_err(RainyError::ValidationError)?;
        request.stream = Some(true);
        self.execute_once(|| async {
            let response = self
                .send_request(self.json_request(Method::POST, "/responses", &request)?)
                .await?;
            self.handle_responses_stream_response(response).await
        })
        .await
    }

    /// Performs a simple single-prompt Chat request and extracts text.
    pub async fn simple_chat(
        &self,
        model: impl Into<String>,
        prompt: impl Into<String>,
    ) -> Result<String> {
        let response = self
            .create_chat_completion(ChatCompletionRequest::new(
                model,
                vec![ChatMessage::user(prompt)],
            ))
            .await?;
        Ok(response
            .choices
            .into_iter()
            .next()
            .map(|choice| choice.message.content)
            .unwrap_or_default())
    }

    /// Bounds and decodes a successful response or maps a safe error.
    pub(crate) async fn handle_response<T: DeserializeOwned>(
        &self,
        response: Response,
    ) -> Result<T> {
        let status = response.status();
        let request_id = header_string(response.headers(), "x-request-id");
        let retry_after = retry_after_seconds(response.headers());
        if status.is_success() {
            let body = read_limited_response_body(response, MAX_RESPONSE_BODY_BYTES).await?;
            serde_json::from_slice(&body).map_err(|error| RainyError::Serialization {
                message: "failed to decode JSON response".to_string(),
                source_error: Some(error.to_string()),
            })
        } else {
            let body = read_limited_response_body(response, MAX_ERROR_BODY_BYTES).await?;
            self.handle_error_text(
                status,
                request_id,
                retry_after,
                String::from_utf8_lossy(&body).into_owned(),
            )
        }
    }

    pub(crate) async fn handle_chat_stream_response(
        &self,
        response: Response,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatStreamEvent>> + Send>>> {
        let response = self.prepare_stream_response(response).await?;
        let stream = parse_sse_stream(response.bytes_stream()).filter_map(|event| async move {
            match event {
                Ok(event) if event.done || event.data.trim().is_empty() => None,
                Ok(event) => match serde_json::from_str::<Value>(event.data.trim()) {
                    Ok(value) => Some(Ok(ChatStreamEvent::from_sse_event(
                        event.event.as_deref(),
                        value,
                    ))),
                    Err(_) => Some(Err(RainyError::Serialization {
                        message: "failed to decode Chat SSE payload".to_string(),
                        source_error: None,
                    })),
                },
                Err(error) => Some(Err(error)),
            }
        });
        Ok(Box::pin(stream))
    }

    pub(crate) async fn handle_responses_stream_response(
        &self,
        response: Response,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ResponsesStreamEvent>> + Send>>> {
        let response = self.prepare_stream_response(response).await?;
        let stream = parse_sse_stream(response.bytes_stream()).filter_map(|event| async move {
            match event {
                Ok(event) if event.done || event.data.trim().is_empty() => None,
                Ok(event) => match serde_json::from_str::<Value>(event.data.trim()) {
                    Ok(value) => Some(Ok(ResponsesEvent::new(event.event, value))),
                    Err(_) => Some(Err(RainyError::Serialization {
                        message: "failed to decode Responses SSE payload".to_string(),
                        source_error: None,
                    })),
                },
                Err(error) => Some(Err(error)),
            }
        });
        Ok(Box::pin(stream))
    }

    pub(crate) async fn handle_anthropic_message_stream_response(
        &self,
        response: Response,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AnthropicMessageStreamEvent>> + Send>>> {
        let response = self.prepare_stream_response(response).await?;
        let stream = parse_sse_stream(response.bytes_stream()).filter_map(|event| async move {
            match event {
                Ok(event) if event.done || event.data.trim().is_empty() => None,
                Ok(event) => match serde_json::from_str::<Value>(event.data.trim()) {
                    Ok(value) => Some(Ok(AnthropicMessageStreamEvent::new(event.event, value))),
                    Err(_) => Some(Err(RainyError::Serialization {
                        message: "failed to decode Anthropic SSE payload".to_string(),
                        source_error: None,
                    })),
                },
                Err(error) => Some(Err(error)),
            }
        });
        Ok(Box::pin(stream))
    }

    async fn prepare_stream_response(&self, response: Response) -> Result<Response> {
        if response.status().is_success() {
            return Ok(response);
        }
        let status = response.status();
        let request_id = header_string(response.headers(), "x-request-id");
        let retry_after = retry_after_seconds(response.headers());
        let body = read_limited_response_body(response, MAX_ERROR_BODY_BYTES).await?;
        self.handle_error_text(
            status,
            request_id,
            retry_after,
            String::from_utf8_lossy(&body).into_owned(),
        )
    }

    fn handle_error_text<T>(
        &self,
        status: reqwest::StatusCode,
        request_id: Option<String>,
        retry_after: Option<u64>,
        text: String,
    ) -> Result<T> {
        let parsed = serde_json::from_str::<Value>(&text).ok();
        let (code, message, details, server_retryable) = parsed
            .as_ref()
            .and_then(extract_error_fields)
            .unwrap_or_else(|| {
                (
                    status
                        .canonical_reason()
                        .unwrap_or("HTTP_ERROR")
                        .to_string(),
                    if text.is_empty() {
                        format!("HTTP {}", status.as_u16())
                    } else {
                        "The service returned an invalid error response".to_string()
                    },
                    None,
                    None,
                )
            });

        self.map_api_error(
            ApiErrorDetails {
                code,
                message,
                details,
                retryable: server_retryable,
                timestamp: None,
                request_id: request_id.clone(),
            },
            status.as_u16(),
            request_id,
            retry_after,
        )
    }

    fn extract_metadata(&self, response: &Response, started: Instant) -> RequestMetadata {
        let headers = response.headers();
        RequestMetadata {
            response_time: Some(started.elapsed().as_millis() as u64),
            provider: header_string(headers, "x-provider"),
            tokens_used: header_string(headers, "x-tokens-used")
                .and_then(|value| value.parse().ok()),
            credits_used: header_string(headers, "x-credits-used")
                .and_then(|value| value.parse().ok()),
            credits_remaining: header_string(headers, "x-credits-remaining")
                .and_then(|value| value.parse().ok()),
            request_id: header_string(headers, "x-request-id"),
            compat_warnings: header_string(headers, "x-rainy-compat-warnings")
                .and_then(|value| value.parse().ok()),
            response_mode: header_string(headers, "x-rainy-response-mode"),
            billing_plan: header_string(headers, "x-rainy-billing-plan"),
            rainy_credits_charged: header_string(headers, "x-rainy-credits-charged")
                .and_then(|value| value.parse().ok()),
            rainy_daily_credits_remaining: header_string(
                headers,
                "x-rainy-daily-credits-remaining",
            ),
            rainy_sanitized_params: header_string(headers, "x-rainy-sanitized-params"),
            rainy_billing_adjustment: header_string(headers, "x-rainy-billing-adjustment"),
            rainy_billing_outstanding_credits: header_string(
                headers,
                "x-rainy-billing-outstanding-credits",
            )
            .and_then(|value| value.parse().ok()),
        }
    }

    fn map_api_error<T>(
        &self,
        error: ApiErrorDetails,
        status_code: u16,
        request_id: Option<String>,
        retry_after_header: Option<u64>,
    ) -> Result<T> {
        let retryable = error.retryable.unwrap_or(status_code >= 500);
        let secret = self.auth_config.api_key.expose_secret();
        let code = safe_error_message(&redact_secret(&error.code, secret));
        let message = safe_error_message(&redact_secret(&error.message, secret));
        let details = error.details.map(|details| redact_json(details, secret));
        if status_code == 401
            || matches!(
                code.as_str(),
                "INVALID_API_KEY" | "EXPIRED_API_KEY" | "UNAUTHORIZED" | "INVALID_TOKEN"
            )
        {
            return Err(RainyError::Authentication {
                code,
                message,
                retryable: false,
            });
        }
        if status_code == 403 || matches!(code.as_str(), "FORBIDDEN" | "ACCESS_DENIED") {
            return Err(RainyError::AccessDenied {
                code,
                message,
                details,
            });
        }
        if status_code == 413 {
            return Err(RainyError::PayloadTooLarge {
                message: "the service rejected the request as too large".to_string(),
                max_bytes: MODEL_REQUEST_BODY_BYTES,
            });
        }
        if status_code == 429 || code == "RATE_LIMIT_EXCEEDED" {
            let retry_after = details
                .as_ref()
                .and_then(|details| details.get("retry_after"))
                .and_then(Value::as_u64)
                .map(|value| value.min(86_400))
                .or(retry_after_header);
            return Err(RainyError::RateLimit {
                code,
                message,
                retry_after,
                current_usage: None,
            });
        }
        if code == "INSUFFICIENT_CREDITS" {
            let detail_ref = details.as_ref();
            return Err(RainyError::InsufficientCredits {
                code,
                message,
                current_credits: detail_ref
                    .and_then(|value| value.get("current_credits"))
                    .and_then(Value::as_f64)
                    .unwrap_or_default(),
                estimated_cost: detail_ref
                    .and_then(|value| value.get("estimated_cost"))
                    .and_then(Value::as_f64)
                    .unwrap_or_default(),
                reset_date: detail_ref
                    .and_then(|value| value.get("reset_date"))
                    .and_then(Value::as_str)
                    .map(ToOwned::to_owned),
            });
        }
        if matches!(
            code.as_str(),
            "INVALID_REQUEST" | "MISSING_REQUIRED_FIELD" | "INVALID_MODEL"
        ) || (400..500).contains(&status_code)
        {
            return Err(RainyError::InvalidRequest {
                code,
                message,
                details,
            });
        }
        if matches!(code.as_str(), "PROVIDER_ERROR" | "PROVIDER_UNAVAILABLE") {
            return Err(RainyError::Provider {
                code,
                message,
                provider: "upstream".to_string(),
                retryable,
            });
        }
        Err(RainyError::Api {
            code,
            message,
            status_code,
            retryable,
            request_id,
        })
    }

    /// Returns the configured authentication settings.
    pub fn auth_config(&self) -> &AuthConfig {
        &self.auth_config
    }

    /// Returns the root base URL.
    pub fn base_url(&self) -> &str {
        &self.auth_config.base_url
    }

    /// Returns the effective versioned API base URL.
    pub fn api_base_url(&self) -> String {
        if let Some(value) = &self.auth_config.api_base_url {
            return value.clone();
        }
        let base = self.auth_config.base_url.trim_end_matches('/');
        let has_path = url::Url::parse(base)
            .ok()
            .is_some_and(|url| url.path() != "" && url.path() != "/");
        if has_path {
            base.to_string()
        } else {
            format!("{base}/api/v1")
        }
    }

    /// Compatibility helper for older endpoint modules.
    #[cfg(feature = "legacy")]
    pub(crate) async fn make_request<T: DeserializeOwned>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<Value>,
    ) -> Result<T> {
        self.execute_once(|| async {
            let request = match body.as_ref() {
                Some(body) => self.json_request(method.clone(), endpoint, body)?,
                None => self.api_request(method.clone(), endpoint),
            };
            let response = self.send_request(request).await?;
            self.handle_response(response).await
        })
        .await
    }
}

fn header_string(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(ToOwned::to_owned)
}

fn safe_error_message(message: &str) -> String {
    let mut output = message
        .chars()
        .filter(|character| !character.is_control() || matches!(character, '\n' | '\t'))
        .collect::<String>();
    if output.chars().count() > 1024 {
        output = output.chars().take(1024).collect();
        output.push('…');
    }
    if output.is_empty() {
        "The service returned an error".to_string()
    } else {
        output
    }
}

fn redact_secret(value: &str, secret: &str) -> String {
    if secret.is_empty() {
        value.to_string()
    } else {
        value.replace(secret, "<redacted>")
    }
}

fn redact_json(value: Value, secret: &str) -> Value {
    match value {
        Value::String(value) => Value::String(redact_secret(&value, secret)),
        Value::Array(values) => Value::Array(
            values
                .into_iter()
                .map(|value| redact_json(value, secret))
                .collect(),
        ),
        Value::Object(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| (key, redact_json(value, secret)))
                .collect(),
        ),
        value => value,
    }
}

fn extract_error_fields(value: &Value) -> Option<(String, String, Option<Value>, Option<bool>)> {
    let error = value.get("error").unwrap_or(value);
    let code = error
        .get("code")
        .and_then(Value::as_str)
        .or_else(|| error.get("type").and_then(Value::as_str))
        .or_else(|| value.get("code").and_then(Value::as_str))
        .unwrap_or("API_ERROR")
        .to_string();
    let message = error
        .get("message")
        .and_then(Value::as_str)
        .or_else(|| value.get("message").and_then(Value::as_str))
        .unwrap_or("The service returned an error")
        .to_string();
    let details = error
        .get("details")
        .cloned()
        .or_else(|| value.get("details").cloned());
    let retryable = error
        .get("retryable")
        .and_then(Value::as_bool)
        .or_else(|| value.get("retryable").and_then(Value::as_bool));
    Some((code, message, details, retryable))
}

fn decode_model_list(value: Value) -> Result<ModelList> {
    let data = value.get("data").cloned().unwrap_or(value);
    let data = if data.get("data").is_some() {
        data.get("data").cloned().unwrap_or(data)
    } else {
        data
    };
    serde_json::from_value(data).map_err(|error| RainyError::Serialization {
        message: "failed to decode model list".to_string(),
        source_error: Some(error.to_string()),
    })
}

fn decode_catalog(value: Value) -> Result<Vec<ModelCatalogItem>> {
    let data = value.get("data").cloned().unwrap_or(value);
    let data = if data.is_array() {
        data
    } else {
        data.get("data").cloned().unwrap_or(data)
    };
    serde_json::from_value(data).map_err(|error| RainyError::Serialization {
        message: "failed to decode model catalog".to_string(),
        source_error: Some(error.to_string()),
    })
}

impl std::fmt::Debug for RainyClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RainyClient")
            .field("base_url", &safe_url_label(&self.auth_config.base_url))
            .field(
                "api_base_url",
                &self.auth_config.api_base_url.as_deref().map(safe_url_label),
            )
            .field("timeout", &self.auth_config.timeout_seconds)
            .field("max_retries", &self.retry_config.max_retries)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key() -> String {
        format!("ra-{}", "a".repeat(48))
    }

    #[test]
    fn default_routes_use_rainy_api_prefix() {
        let client = RainyClient::with_config(
            AuthConfig::new(key()).with_base_url("https://gateway.example.com/"),
        )
        .unwrap();
        assert_eq!(
            client.api_v1_url("responses"),
            "https://gateway.example.com/api/v1/responses"
        );
    }

    #[test]
    fn custom_versioned_base_url_is_used_directly() {
        let client = RainyClient::with_config(
            AuthConfig::new("sk-compatible")
                .with_base_url("https://example-compatible-provider.com/v1"),
        )
        .unwrap();
        assert_eq!(
            client.api_v1_url("/responses"),
            "https://example-compatible-provider.com/v1/responses"
        );
    }

    #[test]
    fn extracts_nested_error_details_and_retryability() {
        let value = serde_json::json!({
            "error": {
                "code": "RATE_LIMIT_EXCEEDED",
                "message": "slow down",
                "retryable": true,
                "details": {"retry_after": 2}
            }
        });
        let (code, message, details, retryable) = extract_error_fields(&value).unwrap();
        assert_eq!(code, "RATE_LIMIT_EXCEEDED");
        assert_eq!(message, "slow down");
        assert_eq!(details.unwrap()["retry_after"], 2);
        assert_eq!(retryable, Some(true));
    }
}
