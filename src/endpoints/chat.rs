//! Full OpenAI-compatible Chat message operations.

use crate::client::RainyClient;
use crate::error::Result;
use crate::models::{
    ChatCompletionStreamResponse, ChatStreamEvent, OpenAIChatCompletionRequest,
    OpenAIChatCompletionResponse, RainyEnvelope,
};
use futures::{Stream, StreamExt};
use std::pin::Pin;

impl RainyClient {
    /// Creates a Chat completion using full OpenAI-compatible messages.
    pub async fn create_openai_chat_completion(
        &self,
        request: OpenAIChatCompletionRequest,
    ) -> Result<OpenAIChatCompletionResponse> {
        request
            .validate_openai_compatibility()
            .map_err(crate::error::RainyError::ValidationError)?;
        self.wait_for_slot().await;
        let response = self
            .send_request(self.json_request(
                reqwest::Method::POST,
                "/chat/completions",
                &request,
            )?)
            .await?;
        self.handle_response(response).await
    }

    /// Creates a full-message Chat completion in Rainy envelope mode.
    pub async fn openai_chat_completion_envelope(
        &self,
        request: OpenAIChatCompletionRequest,
    ) -> Result<RainyEnvelope<OpenAIChatCompletionResponse>> {
        request
            .validate_openai_compatibility()
            .map_err(crate::error::RainyError::ValidationError)?;
        self.wait_for_slot().await;
        let response = self
            .send_request(
                self.json_request(reqwest::Method::POST, "/chat/completions", &request)?
                    .header("X-Rainy-Response-Mode", "envelope"),
            )
            .await?;
        self.handle_response(response).await
    }

    /// Creates a full-message streaming Chat completion and returns chunks.
    pub async fn create_openai_chat_completion_stream(
        &self,
        mut request: OpenAIChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionStreamResponse>> + Send>>> {
        request
            .validate_openai_compatibility()
            .map_err(crate::error::RainyError::ValidationError)?;
        request.stream = Some(true);
        self.wait_for_slot().await;
        let response = self
            .send_request(self.json_request(
                reqwest::Method::POST,
                "/chat/completions",
                &request,
            )?)
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
        Ok(Box::pin(chunks))
    }

    /// Creates a full-message streaming Chat completion with typed events.
    pub async fn create_openai_chat_completion_stream_events(
        &self,
        mut request: OpenAIChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatStreamEvent>> + Send>>> {
        request
            .validate_openai_compatibility()
            .map_err(crate::error::RainyError::ValidationError)?;
        request.stream = Some(true);
        self.wait_for_slot().await;
        let response = self
            .send_request(self.json_request(
                reqwest::Method::POST,
                "/chat/completions",
                &request,
            )?)
            .await?;
        self.handle_chat_stream_response(response).await
    }
}
