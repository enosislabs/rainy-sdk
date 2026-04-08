use crate::client::RainyClient;
use crate::error::Result;
use crate::models::{
    ChatCompletionRequest, ChatCompletionResponse, ChatCompletionStreamResponse, ChatStreamEvent,
    OpenAIChatCompletionRequest, OpenAIChatCompletionResponse, RainyEnvelope,
};
use futures::{Stream, StreamExt};
use std::pin::Pin;

impl RainyClient {
    /// Create a chat completion
    ///
    /// This endpoint sends a chat completion request to the Rainy API.
    ///
    /// # Arguments
    ///
    /// * `request` - The chat completion request parameters
    ///
    /// # Returns
    ///
    /// Returns the chat completion response from the AI model.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::{RainyClient, ChatCompletionRequest, ChatMessage, MessageRole};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("user-api-key")?;
    ///
    /// let messages = vec![
    ///     ChatMessage::user("Hello, how are you?"),
    /// ];
    ///
    /// let request = ChatCompletionRequest::new("gemini-pro", messages)
    ///     .with_max_tokens(150)
    ///     .with_temperature(0.7);
    ///
    /// let response = client.create_chat_completion(request).await?;
    ///
    /// if let Some(choice) = response.choices.first() {
    ///     println!("Response: {}", choice.message.content);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_chat_completion(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<ChatCompletionResponse> {
        let body = serde_json::to_value(request)?;
        self.make_request(reqwest::Method::POST, "/chat/completions", Some(body))
            .await
    }

    /// Create an OpenAI-compatible chat completion with full tool-call replay support.
    ///
    /// This variant accepts the complete OpenAI message shape, including:
    /// assistant `tool_calls`, `tool` role messages, multimodal content parts, and
    /// provider-specific metadata such as thought signatures.
    pub async fn create_openai_chat_completion(
        &self,
        request: OpenAIChatCompletionRequest,
    ) -> Result<OpenAIChatCompletionResponse> {
        let body = serde_json::to_value(request)?;
        self.make_request(reqwest::Method::POST, "/chat/completions", Some(body))
            .await
    }

    /// Create a chat completion in envelope mode.
    pub async fn create_chat_completion_envelope(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<RainyEnvelope<ChatCompletionResponse>> {
        let (response, _) = self.chat_completion_envelope(request).await?;
        Ok(response)
    }

    /// Create an OpenAI-compatible chat completion in envelope mode.
    pub async fn create_openai_chat_completion_envelope(
        &self,
        request: OpenAIChatCompletionRequest,
    ) -> Result<RainyEnvelope<OpenAIChatCompletionResponse>> {
        let (response, _) = self.openai_chat_completion_envelope(request).await?;
        Ok(response)
    }

    /// Create a chat completion with streaming
    ///
    /// This method provides streaming support for chat completions.
    ///
    /// # Arguments
    ///
    /// * `request` - The chat completion request parameters
    ///
    /// # Returns
    ///
    /// Returns a stream of chat completion responses.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::{RainyClient, ChatCompletionRequest, ChatMessage, MessageRole};
    /// # use futures::StreamExt;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("user-api-key")?;
    ///
    /// let messages = vec![
    ///     ChatMessage::user("Tell me a story"),
    /// ];
    ///
    /// let request = ChatCompletionRequest::new("llama-3.1-8b-instant", messages)
    ///     .with_max_tokens(500)
    ///     .with_temperature(0.8)
    ///     .with_stream(true);
    ///
    /// let mut stream = client.create_chat_completion_stream(request).await?;
    ///
    /// while let Some(chunk) = stream.next().await {
    ///     match chunk {
    ///         Ok(response) => {
    ///             if let Some(choice) = response.choices.first() {
    ///                 if let Some(content) = &choice.delta.content {
    ///                     print!("{}", content);
    ///                 }
    ///             }
    ///         }
    ///         Err(e) => eprintln!("Error: {}", e),
    ///     }
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_chat_completion_stream(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionStreamResponse>> + Send>>> {
        self.chat_completion_stream(request).await
    }

    /// Create a streaming OpenAI-compatible chat completion.
    ///
    /// This method uses the same `/api/v1/chat/completions` route but accepts the full
    /// OpenAI-compatible message format so callers can replay tool history without a
    /// separate compatibility bridge.
    pub async fn create_openai_chat_completion_stream(
        &self,
        request: OpenAIChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionStreamResponse>> + Send>>> {
        let mut request_with_stream = request;
        request_with_stream.stream = Some(true);

        let url = self.api_v1_url("/chat/completions");

        let response = self
            .http_client()
            .post(&url)
            .json(&request_with_stream)
            .send()
            .await?;

        let events = self.handle_chat_stream_response(response).await?;
        let stream = events.filter_map(|event| async move {
            match event {
                Ok(ChatStreamEvent::Chunk(chunk)) => Some(Ok(chunk)),
                Ok(ChatStreamEvent::Billing(_)) | Ok(ChatStreamEvent::Raw(_)) => None,
                Err(error) => Some(Err(error)),
            }
        });
        Ok(Box::pin(stream))
    }

    /// Create a streaming chat completion returning typed stream events.
    pub async fn create_chat_completion_stream_events(
        &self,
        request: ChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatStreamEvent>> + Send>>> {
        self.chat_completion_stream_events(request).await
    }

    /// Create a streaming OpenAI-compatible chat completion returning typed stream events.
    pub async fn create_openai_chat_completion_stream_events(
        &self,
        request: OpenAIChatCompletionRequest,
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatStreamEvent>> + Send>>> {
        let mut request_with_stream = request;
        request_with_stream.stream = Some(true);

        let url = self.api_v1_url("/chat/completions");
        let response = self
            .http_client()
            .post(&url)
            .json(&request_with_stream)
            .send()
            .await?;

        self.handle_chat_stream_response(response).await
    }
}
