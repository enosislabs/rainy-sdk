//! Anthropic-compatible Messages endpoint.

use crate::client::RainyClient;
use crate::error::{RainyError, Result};
use crate::models::{
    AnthropicMessageRequest, AnthropicMessageResponse, AnthropicMessageStreamEvent,
};
use futures::Stream;
use reqwest::Method;
use std::pin::Pin;

/// Anthropic's current stable Messages wire version used when callers do not
/// provide an explicit version.
pub const DEFAULT_ANTHROPIC_VERSION: &str = "2023-06-01";

fn validate_header(value: &str, name: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Err(RainyError::InvalidRequest {
            code: format!("INVALID_{}", name.to_ascii_uppercase().replace('-', "_")),
            message: format!("{name} header cannot be empty"),
            details: None,
        });
    }
    reqwest::header::HeaderValue::from_str(value).map_err(|error| RainyError::InvalidRequest {
        code: format!("INVALID_{}", name.to_ascii_uppercase().replace('-', "_")),
        message: format!("{name} header contains invalid characters"),
        details: Some(serde_json::json!({"error": error.to_string()})),
    })?;
    Ok(())
}

impl RainyClient {
    /// Creates an Anthropic-compatible Messages response.
    ///
    /// The SDK sends the required `anthropic-version: 2023-06-01` header and
    /// uses the same API-key authentication and bounded response handling as
    /// the Chat and Responses endpoints.
    pub async fn create_message(
        &self,
        request: AnthropicMessageRequest,
    ) -> Result<AnthropicMessageResponse> {
        self.create_message_with_headers(request, DEFAULT_ANTHROPIC_VERSION, None)
            .await
    }

    /// Alias for [`Self::create_message`] that makes the provider contract
    /// explicit at call sites.
    pub async fn create_anthropic_message(
        &self,
        request: AnthropicMessageRequest,
    ) -> Result<AnthropicMessageResponse> {
        self.create_message(request).await
    }

    /// Creates a Messages response with explicit Anthropic version and beta
    /// headers.
    pub async fn create_message_with_headers(
        &self,
        request: AnthropicMessageRequest,
        anthropic_version: &str,
        anthropic_beta: Option<&str>,
    ) -> Result<AnthropicMessageResponse> {
        validate_header(anthropic_version, "anthropic-version")?;
        if let Some(beta) = anthropic_beta {
            validate_header(beta, "anthropic-beta")?;
        }

        self.wait_for_slot().await;
        let mut builder = self
            .json_request(Method::POST, "/messages", &request)?
            .header("anthropic-version", anthropic_version);
        if let Some(beta) = anthropic_beta {
            builder = builder.header("anthropic-beta", beta);
        }
        let response = self.send_request(builder).await?;
        self.handle_response(response).await
    }

    /// Creates a streaming Anthropic Messages response using the default wire
    /// version.
    pub async fn create_message_stream(
        &self,
        mut request: AnthropicMessageRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AnthropicMessageStreamEvent>> + Send>>> {
        request.stream = Some(true);
        self.create_message_stream_with_headers(request, DEFAULT_ANTHROPIC_VERSION, None)
            .await
    }

    /// Alias for [`Self::create_message_stream`] with an explicit provider name.
    pub async fn create_anthropic_message_stream(
        &self,
        request: AnthropicMessageRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AnthropicMessageStreamEvent>> + Send>>> {
        self.create_message_stream(request).await
    }

    /// Creates a streaming Messages response with explicit Anthropic headers.
    pub async fn create_message_stream_with_headers(
        &self,
        mut request: AnthropicMessageRequest,
        anthropic_version: &str,
        anthropic_beta: Option<&str>,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AnthropicMessageStreamEvent>> + Send>>> {
        request.stream = Some(true);
        validate_header(anthropic_version, "anthropic-version")?;
        if let Some(beta) = anthropic_beta {
            validate_header(beta, "anthropic-beta")?;
        }

        self.wait_for_slot().await;
        let mut builder = self
            .json_request(Method::POST, "/messages", &request)?
            .header("anthropic-version", anthropic_version)
            .header("accept", "text/event-stream");
        if let Some(beta) = anthropic_beta {
            builder = builder.header("anthropic-beta", beta);
        }
        let response = self.send_request(builder).await?;
        self.handle_anthropic_message_stream_response(response)
            .await
    }
}
