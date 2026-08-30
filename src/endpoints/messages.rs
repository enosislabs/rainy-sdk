//! Native Anthropic-compatible Messages operations.

use crate::client::RainyClient;
use crate::error::Result;
use crate::models::{
    AnthropicMessageRequest, AnthropicMessageResponse, AnthropicMessageStreamEvent,
};
use futures::Stream;
use std::pin::Pin;

/// Default Anthropic API version used by the native Messages protocol.
pub const DEFAULT_ANTHROPIC_VERSION: &str = "2023-06-01";

impl RainyClient {
    /// Creates a native Anthropic Messages response.
    pub async fn create_message(
        &self,
        request: AnthropicMessageRequest,
    ) -> Result<AnthropicMessageResponse> {
        request
            .validate()
            .map_err(crate::error::RainyError::ValidationError)?;
        self.wait_for_slot().await;
        let response = self
            .send_request(self.anthropic_json_request(
                reqwest::Method::POST,
                "/messages",
                &request,
            )?)
            .await?;
        self.handle_response(response).await
    }

    /// Alias for [`Self::create_message`] with the protocol name made explicit.
    pub async fn create_anthropic_message(
        &self,
        request: AnthropicMessageRequest,
    ) -> Result<AnthropicMessageResponse> {
        self.create_message(request).await
    }

    /// Creates a native Anthropic Messages stream.
    pub async fn create_message_stream(
        &self,
        mut request: AnthropicMessageRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AnthropicMessageStreamEvent>> + Send>>> {
        request.stream = Some(true);
        request
            .validate()
            .map_err(crate::error::RainyError::ValidationError)?;
        self.wait_for_slot().await;
        let response = self
            .send_request(
                self.anthropic_json_request(reqwest::Method::POST, "/messages", &request)?
                    .header(reqwest::header::ACCEPT, "text/event-stream"),
            )
            .await?;
        self.handle_anthropic_message_stream_response(response)
            .await
    }

    /// Alias for [`Self::create_message_stream`].
    pub async fn create_anthropic_message_stream(
        &self,
        request: AnthropicMessageRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<AnthropicMessageStreamEvent>> + Send>>> {
        self.create_message_stream(request).await
    }
}
