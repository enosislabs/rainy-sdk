//! Web Research endpoint
//!
//! This endpoint provides web research capabilities via the Rainy API v3 search API.

use crate::{
    error::{RainyError, Result},
    search::{DeepResearchResponse, ResearchConfig},
    RainyClient,
};
use serde::Deserialize;
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
        #[derive(Deserialize)]
        struct SearchResultItem {
            title: Option<String>,
            url: Option<String>,
            content: Option<String>,
            snippet: Option<String>,
        }
        #[derive(Deserialize)]
        struct SearchData {
            results: Vec<SearchResultItem>,
        }
        #[derive(Deserialize)]
        struct SearchEnvelope {
            success: bool,
            data: SearchData,
        }

        let cfg = config.unwrap_or_default();
        let topic = topic.into();
        let url = self.api_v1_url("/search");
        let search_depth = match cfg.depth {
            crate::models::ResearchDepth::Advanced => "advanced",
            _ => "basic",
        };
        let request = json!({
            "query": topic,
            "searchDepth": search_depth,
            "maxResults": cfg.max_sources.min(20),
        });

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

        let envelope: SearchEnvelope = self.handle_response(response).await?;

        let results_json = envelope
            .data
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
            .data
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
            success: envelope.success,
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
}
