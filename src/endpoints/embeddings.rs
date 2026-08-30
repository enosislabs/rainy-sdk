use crate::client::RainyClient;
use crate::error::Result;
use crate::models::{EmbeddingsRequest, EmbeddingsResponse};

impl RainyClient {
    /// Create embeddings through the OpenAI-compatible `/api/v1/embeddings` route.
    pub async fn create_embeddings(
        &self,
        request: EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse> {
        request
            .validate()
            .map_err(crate::error::RainyError::ValidationError)?;
        self.wait_for_slot().await;
        let response = self
            .send_request(self.json_request(reqwest::Method::POST, "/embeddings", &request)?)
            .await?;
        self.handle_response(response).await
    }
}
