use crate::client::RainyClient;
use crate::error::{RainyError, Result};
use crate::models::{ChatCompletionRequest, ChatCompletionResponse};
use futures::Stream;
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
    /// # use rainy_sdk::{RainyClient, AuthConfig, ChatCompletionRequest, ChatMessage, ChatRole};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::new(
    ///     AuthConfig::new().with_api_key("user-api-key")
    /// )?;
    ///
    /// let messages = vec![
    ///     ChatMessage {
    ///         role: ChatRole::User,
    ///         content: "Hello, how are you?".to_string(),
    ///     }
    /// ];
    ///
    /// let request = ChatCompletionRequest {
    ///     model: "gemini-pro".to_string(),
    ///     messages,
    ///     max_tokens: Some(150),
    ///     temperature: Some(0.7),
    ///     stream: None,
    /// };
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
    /// # use rainy_sdk::{RainyClient, AuthConfig, ChatCompletionRequest, ChatMessage, ChatRole};
    /// # use futures::StreamExt;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::new(
    ///     AuthConfig::new().with_api_key("user-api-key")
    /// )?;
    ///
    /// let messages = vec![
    ///     ChatMessage {
    ///         role: ChatRole::User,
    ///         content: "Tell me a story".to_string(),
    ///     }
    /// ];
    ///
    /// let request = ChatCompletionRequest {
    ///     model: "llama-3.1-8b-instant".to_string(),
    ///     messages,
    ///     max_tokens: Some(500),
    ///     temperature: Some(0.8),
    ///     stream: Some(true),
    /// };
    ///
    /// let mut stream = client.create_chat_completion_stream(request).await?;
    ///
    /// while let Some(chunk) = stream.next().await {
    ///     match chunk {
    ///         Ok(response) => {
    ///             if let Some(choice) = response.choices.first() {
    ///                 print!("{}", choice.message.content);
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
    ) -> Result<Pin<Box<dyn Stream<Item = Result<ChatCompletionResponse>> + Send>>> {
        use futures::StreamExt;

        let mut request_with_stream = request;
        request_with_stream.stream = Some(true);

        let url = format!("{}/api/v1/chat/completions", self.config.base_url);
        let headers = self.config.build_headers()?;

        let response = self.http_client
            .post(&url)
            .headers(headers)
            .json(&request_with_stream)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(self.handle_response::<ChatCompletionResponse>(response).await.err().unwrap());
        }

        let stream = response.bytes_stream().map(|chunk| {
            match chunk {
                Ok(bytes) => {
                    // Parse SSE format (simplified)
                    let text = String::from_utf8_lossy(&bytes);
                    if let Some(json_data) = text.strip_prefix("data: ") {
                        if json_data.trim() == "[DONE]" {
                            return Ok(None);
                        }
                        match serde_json::from_str::<ChatCompletionResponse>(json_data) {
                            Ok(response) => Ok(Some(response)),
                            Err(e) => Err(RainyError::Json(e)),
                        }
                    } else {
                        Ok(None)
                    }
                }
                Err(e) => Err(RainyError::Http(e)),
            }
        })
        .filter_map(|result| async move {
            match result {
                Ok(Some(response)) => Some(Ok(response)),
                Ok(None) => None,
                Err(e) => Some(Err(e)),
            }
        });

        Ok(Box::pin(stream))
    }
}
