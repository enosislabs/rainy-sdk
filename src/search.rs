//! Web Search Module
//!
//! This module provides types and functionality for web search via Tavily API.
//! Requires a Cowork plan with web_research feature enabled.

use serde::{Deserialize, Serialize};

/// Individual search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    /// Page title
    pub title: String,
    /// Page URL
    pub url: String,
    /// Relevant content snippet
    pub content: String,
    /// Relevance score (0.0 - 1.0)
    #[serde(default)]
    pub score: f32,
    /// Publication date if available
    #[serde(default)]
    pub published_date: Option<String>,
}

/// Web search response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResponse {
    /// Original search query
    pub query: String,
    /// Search results
    pub results: Vec<SearchResult>,
    /// AI-generated answer if requested
    #[serde(default)]
    pub answer: Option<String>,
    /// Response time in milliseconds
    #[serde(default)]
    pub response_time: u64,
}

/// Options for web search
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SearchOptions {
    /// Search depth: "basic" (faster) or "advanced" (more thorough)
    #[serde(default = "default_search_depth")]
    pub search_depth: String,
    /// Maximum number of results (1-20)
    #[serde(default = "default_max_results")]
    pub max_results: u32,
    /// Include AI-generated answer
    #[serde(default)]
    pub include_answer: bool,
    /// Include images in results
    #[serde(default)]
    pub include_images: bool,
    /// Domains to include in search
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_domains: Option<Vec<String>>,
    /// Domains to exclude from search
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude_domains: Option<Vec<String>>,
}

fn default_search_depth() -> String {
    "basic".to_string()
}

fn default_max_results() -> u32 {
    10
}

impl SearchOptions {
    /// Create options for basic search
    pub fn basic() -> Self {
        Self {
            search_depth: "basic".to_string(),
            max_results: 10,
            ..Default::default()
        }
    }

    /// Create options for advanced search with AI answer
    pub fn advanced() -> Self {
        Self {
            search_depth: "advanced".to_string(),
            max_results: 10,
            include_answer: true,
            ..Default::default()
        }
    }

    /// Set maximum results
    pub fn with_max_results(mut self, max: u32) -> Self {
        self.max_results = max.min(20);
        self
    }

    /// Enable AI-generated answer
    pub fn with_answer(mut self) -> Self {
        self.include_answer = true;
        self
    }

    /// Limit search to specific domains
    pub fn with_domains(mut self, domains: Vec<String>) -> Self {
        self.include_domains = Some(domains);
        self
    }

    /// Exclude specific domains
    pub fn without_domains(mut self, domains: Vec<String>) -> Self {
        self.exclude_domains = Some(domains);
        self
    }
}

/// Request body for web search
#[derive(Debug, Clone, Serialize)]
pub(crate) struct SearchRequest {
    pub query: String,
    #[serde(rename = "searchDepth")]
    pub search_depth: String,
    #[serde(rename = "maxResults")]
    pub max_results: u32,
    #[serde(rename = "includeAnswer")]
    pub include_answer: bool,
    #[serde(rename = "includeImages")]
    pub include_images: bool,
    #[serde(rename = "includeDomains", skip_serializing_if = "Option::is_none")]
    pub include_domains: Option<Vec<String>>,
    #[serde(rename = "excludeDomains", skip_serializing_if = "Option::is_none")]
    pub exclude_domains: Option<Vec<String>>,
}

impl SearchRequest {
    pub fn new(query: String, options: &SearchOptions) -> Self {
        Self {
            query,
            search_depth: options.search_depth.clone(),
            max_results: options.max_results,
            include_answer: options.include_answer,
            include_images: options.include_images,
            include_domains: options.include_domains.clone(),
            exclude_domains: options.exclude_domains.clone(),
        }
    }
}

/// Extracted content from a URL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedContent {
    /// Source URL
    pub url: String,
    /// Raw content extracted
    pub raw_content: String,
}

/// Failed extraction result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailedExtraction {
    /// URL that failed
    pub url: String,
    /// Error message
    pub error: String,
}

/// Response from content extraction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractResponse {
    /// Successfully extracted content
    pub results: Vec<ExtractedContent>,
    /// Failed extractions
    #[serde(default)]
    pub failed_results: Vec<FailedExtraction>,
}

/// Request body for content extraction
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ExtractRequest {
    pub urls: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_options_basic() {
        let opts = SearchOptions::basic();
        assert_eq!(opts.search_depth, "basic");
        assert_eq!(opts.max_results, 10);
        assert!(!opts.include_answer);
    }

    #[test]
    fn test_search_options_advanced() {
        let opts = SearchOptions::advanced();
        assert_eq!(opts.search_depth, "advanced");
        assert!(opts.include_answer);
    }

    #[test]
    fn test_search_options_builder() {
        let opts = SearchOptions::basic()
            .with_max_results(5)
            .with_answer()
            .with_domains(vec!["example.com".to_string()]);

        assert_eq!(opts.max_results, 5);
        assert!(opts.include_answer);
        assert!(opts.include_domains.is_some());
    }
}
