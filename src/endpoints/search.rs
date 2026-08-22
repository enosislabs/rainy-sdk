//! Web Research endpoint
//!
//! This endpoint provides web research capabilities via the Rainy API v3 search API.

use crate::{
    RainyClient,
    error::Result,
    search::{DeepResearchResponse, ResearchConfig, SearchExtractResponse, SearchResponse},
};
use serde_json::json;

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
        let topic = topic.into();
        let search_depth = Some(cfg.depth);
        let max_results = Some(cfg.max_sources.min(20));
        let envelope = self.search(topic, search_depth, max_results).await?;

        let results_json = envelope
            .results
            .iter()
            .map(|item| {
                json!({
                    "title": item.title,
                    "url": item.url,
                    "snippet": item.snippet.as_ref().or(item.content.as_ref()),
                })
            })
            .collect::<Vec<_>>();

        let synthesized_content = envelope
            .results
            .iter()
            .enumerate()
            .map(|(idx, item)| {
                let title = item.title.as_deref().unwrap_or("Untitled");
                let url = item.url.as_deref().unwrap_or("");
                let snippet = item
                    .snippet
                    .as_deref()
                    .or(item.content.as_deref())
                    .unwrap_or("");
                format!("{}. {}\n{}\n{}", idx + 1, title, url, snippet)
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(DeepResearchResponse {
            success: true,
            mode: "sync".to_string(),
            result: Some(json!({
                "content": synthesized_content,
                "results": results_json,
            })),
            task_id: None,
            generated_at: None,
            provider: Some("tavily".to_string()),
            message: None,
        })
    }

    /// Performs a native `/api/v1/search` query.
    pub async fn search(
        &self,
        query: impl Into<String>,
        depth: Option<crate::models::ResearchDepth>,
        max_results: Option<u32>,
    ) -> Result<SearchResponse> {
        #[derive(serde::Deserialize)]
        struct SearchEnvelope {
            data: SearchResponse,
        }

        let search_depth = match depth.unwrap_or(crate::models::ResearchDepth::Basic) {
            crate::models::ResearchDepth::Advanced => "advanced",
            _ => "basic",
        };

        let request = json!({
            "query": query.into(),
            "searchDepth": search_depth,
            "maxResults": max_results.unwrap_or(10).clamp(1, 20),
        });

        self.wait_for_slot().await;
        let response = self
            .send_request(
                self.api_request(reqwest::Method::POST, "/search")
                    .json(&request),
            )
            .await?;

        let envelope: SearchEnvelope = self.handle_response(response).await?;
        Ok(envelope.data)
    }

    /// Performs a native `/api/v1/search/extract` request.
    pub async fn search_extract(&self, urls: Vec<String>) -> Result<SearchExtractResponse> {
        #[derive(serde::Deserialize)]
        struct ExtractEnvelope {
            data: SearchExtractResponse,
        }

        let request = json!({
            "urls": urls,
        });

        self.wait_for_slot().await;
        let response = self
            .send_request(
                self.api_request(reqwest::Method::POST, "/search/extract")
                    .json(&request),
            )
            .await?;

        let envelope: ExtractEnvelope = self.handle_response(response).await?;
        Ok(envelope.data)
    }
}
