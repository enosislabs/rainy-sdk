//! Web Research endpoint
//!
//! This endpoint provides deep web research capabilities via Exa/Tavily.
//! Requires a Cowork plan with web_research feature enabled.

use crate::{
    error::{RainyError, Result},
    search::{DeepResearchResponse, ResearchConfig, ResearchRequest},
    RainyClient,
};

impl RainyClient {
    /// Perform deep web research on a topic.
    ///
    /// This method leverages the Rainy Agent Network to perform comprehensive
    /// web research using providers like Exa or Tavily.
    ///
    /// # Arguments
    ///
    /// * `topic` - The research topic or question.
    /// * `config` - Research configuration (provider, depth, etc.)
    ///
    /// # Returns
    ///
    /// A `Result` containing `DeepResearchResponse` on success.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::{RainyClient, search::ResearchConfig, models::{ResearchProvider, ResearchDepth}};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("your-api-key")?;
    ///
    /// // Basic research
    /// let response = client.research("Latest Rust features", None).await?;
    /// if let Some(content) = response.result {
    ///     println!("Report: {}", content);
    /// }
    ///
    /// // Advanced deep research with Exa
    /// let config = ResearchConfig::new()
    ///     .with_provider(ResearchProvider::Exa)
    ///     .with_depth(ResearchDepth::Advanced);
    ///
    /// let response = client.research("Quantum Computing advances", Some(config)).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn research(
        &self,
        topic: impl Into<String>,
        config: Option<ResearchConfig>,
    ) -> Result<DeepResearchResponse> {
        let cfg = config.unwrap_or_default();
        let request = ResearchRequest::new(topic.into(), &cfg);

        // Note: The endpoint is /agents/research based on previous analysis
        let url = format!("{}/agents/research", self.auth_config().base_url);

        let response = self
            .http_client()
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| RainyError::Network {
                message: e.to_string(),
                retryable: true,
                source_error: Some(e.to_string()),
            })?;

        if response.status().as_u16() == 403 {
            return Err(RainyError::Authentication {
                code: "FEATURE_NOT_AVAILABLE".to_string(),
                message: "Research feature requires a valid subscription".to_string(),
                retryable: false,
            });
        }

        self.handle_response(response).await
    }
}
