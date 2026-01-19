//! Web Search endpoint for Tavily-powered web research
//!
//! This endpoint provides web search and content extraction capabilities.
//! Requires a Cowork plan with web_research feature enabled.

use crate::{
    error::{RainyError, Result},
    search::{ExtractRequest, ExtractResponse, SearchOptions, SearchRequest, SearchResponse},
    RainyClient,
};

impl RainyClient {
    /// Perform a web search using Tavily API.
    ///
    /// This method requires a Cowork plan with web_research feature enabled.
    /// It searches the web and returns relevant results with optional AI-generated answer.
    ///
    /// # Arguments
    ///
    /// * `query` - The search query string.
    /// * `options` - Search options (depth, max results, domains, etc.)
    ///
    /// # Returns
    ///
    /// A `Result` containing `SearchResponse` on success, or `RainyError` on failure.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::{RainyClient, search::{SearchOptions}};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("your-api-key")?;
    ///
    /// // Basic search
    /// let results = client.web_search("Rust programming language", None).await?;
    /// for result in &results.results {
    ///     println!("{}: {}", result.title, result.url);
    /// }
    ///
    /// // Advanced search with AI answer
    /// let options = SearchOptions::advanced()
    ///     .with_max_results(5)
    ///     .with_domains(vec!["rust-lang.org".to_string()]);
    /// let results = client.web_search("Rust async tutorial", Some(options)).await?;
    ///
    /// if let Some(answer) = &results.answer {
    ///     println!("AI Answer: {}", answer);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn web_search(
        &self,
        query: impl Into<String>,
        options: Option<SearchOptions>,
    ) -> Result<SearchResponse> {
        let opts = options.unwrap_or_else(SearchOptions::basic);
        let request = SearchRequest::new(query.into(), &opts);

        let url = format!("{}/api/v1/search", self.auth_config().base_url);

        let response = self
            .http_client()
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| RainyError::NetworkError(e.to_string()))?;

        if response.status().as_u16() == 403 {
            return Err(RainyError::FeatureNotAvailable {
                feature: "web_research".to_string(),
                message: "Web search requires a Cowork plan with web_research enabled".to_string(),
            });
        }

        self.handle_response(response).await
    }

    /// Extract content from URLs using Tavily API.
    ///
    /// This method fetches and extracts the main content from a list of URLs.
    /// Requires a Cowork plan with web_research feature enabled.
    ///
    /// # Arguments
    ///
    /// * `urls` - Vector of URLs to extract content from (max 10).
    ///
    /// # Returns
    ///
    /// A `Result` containing `ExtractResponse` with successful and failed extractions.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::RainyClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("your-api-key")?;
    ///
    /// let response = client.extract_content(vec![
    ///     "https://www.rust-lang.org/".to_string(),
    ///     "https://docs.rs/".to_string(),
    /// ]).await?;
    ///
    /// for extracted in &response.results {
    ///     println!("URL: {}", extracted.url);
    ///     println!("Content length: {} bytes", extracted.raw_content.len());
    /// }
    ///
    /// if !response.failed_results.is_empty() {
    ///     println!("Failed: {} URLs", response.failed_results.len());
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn extract_content(&self, urls: Vec<String>) -> Result<ExtractResponse> {
        if urls.is_empty() {
            return Err(RainyError::ValidationError(
                "At least one URL is required".to_string(),
            ));
        }

        if urls.len() > 10 {
            return Err(RainyError::ValidationError(
                "Maximum 10 URLs allowed per request".to_string(),
            ));
        }

        let request = ExtractRequest { urls };
        let url = format!("{}/api/v1/search/extract", self.auth_config().base_url);

        let response = self
            .http_client()
            .post(&url)
            .json(&request)
            .send()
            .await
            .map_err(|e| RainyError::NetworkError(e.to_string()))?;

        if response.status().as_u16() == 403 {
            return Err(RainyError::FeatureNotAvailable {
                feature: "web_research".to_string(),
                message: "Content extraction requires a Cowork plan with web_research enabled"
                    .to_string(),
            });
        }

        self.handle_response(response).await
    }

    /// Check if web search feature is available for the current plan.
    ///
    /// This is a convenience method that checks the Cowork capabilities.
    pub async fn can_web_search(&self) -> bool {
        self.can_use_feature("web_research").await
    }
}
