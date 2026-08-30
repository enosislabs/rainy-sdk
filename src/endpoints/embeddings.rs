use crate::client::RainyClient;
use crate::error::Result;
use crate::models::{EmbeddingsRequest, EmbeddingsResponse};

impl RainyClient {
    /// Create embeddings through the OpenAI-compatible `/api/v1/embeddings` route.
    pub async fn create_embeddings(
        &self,
        request: EmbeddingsRequest,
    ) -> Result<EmbeddingsResponse> {
        self.make_request(
            reqwest::Method::POST,
            "/embeddings",
            Some(serde_json::to_value(request)?),
        )
        .await
    }
}
