//! Web Research Module
//!
//! This module provides types and functionality for web research via Rainy API v2.
//! Supports multiple providers (Exa, Tavily) and configurable search depth.

use crate::models::{ResearchDepth, ResearchProvider};
use serde::{Deserialize, Serialize};

/// Thinking level for Gemini 3 models
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingLevel {
    /// Minimum reasoning depth
    Minimal,
    /// Fast reasoning depth
    Low,
    /// Balanced reasoning depth
    Medium,
    /// Maximum reasoning depth
    High,
}

/// Options for configuring a web research request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchConfig {
    /// The search provider to use
    #[serde(default)]
    pub provider: ResearchProvider,
    /// The depth of the search
    #[serde(default)]
    pub depth: ResearchDepth,
    /// Maximum number of sources to include
    #[serde(default = "default_max_sources")]
    pub max_sources: u32,
    /// Whether to include images in the results
    #[serde(default)]
    pub include_images: bool,
    /// Process the request asynchronously
    #[serde(default)]
    pub async_mode: bool,
    /// The specific AI model to use for analysis (e.g. "gemini-2.0-flash-exp")
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// The thinking level for Gemini 3 models
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        rename = "thinkingLevel"
    )]
    pub thinking_level: Option<ThinkingLevel>,
}

fn default_max_sources() -> u32 {
    10
}

impl Default for ResearchConfig {
    fn default() -> Self {
        Self {
            provider: ResearchProvider::default(),
            depth: ResearchDepth::default(),
            max_sources: 10,
            include_images: false,
            async_mode: false,
            model: None,
            thinking_level: None,
        }
    }
}

impl ResearchConfig {
    /// Create a new configuration with default settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the search provider
    pub fn with_provider(mut self, provider: ResearchProvider) -> Self {
        self.provider = provider;
        self
    }

    /// Set the search depth
    pub fn with_depth(mut self, depth: ResearchDepth) -> Self {
        self.depth = depth;
        self
    }

    /// Set maximum sources
    pub fn with_max_sources(mut self, max: u32) -> Self {
        self.max_sources = max;
        self
    }

    /// Set the request to be processed asynchronously
    pub fn with_async(mut self, async_mode: bool) -> Self {
        self.async_mode = async_mode;
        self
    }

    /// Set the specific AI model
    pub fn with_model(mut self, model: impl Into<String>) -> Self {
        self.model = Some(model.into());
        self
    }

    /// Set the thinking level (Gemini 3 only)
    pub fn with_thinking_level(mut self, level: ThinkingLevel) -> Self {
        self.thinking_level = Some(level);
        self
    }
}

/// Request body for web research
#[derive(Debug, Clone, Serialize)]
pub(crate) struct ResearchRequest {
    pub topic: String,
    pub provider: ResearchProvider,
    pub depth: ResearchDepth,
    #[serde(rename = "maxSources")]
    pub max_sources: u32,
    #[serde(rename = "async")]
    pub async_mode: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "thinkingLevel")]
    pub thinking_level: Option<ThinkingLevel>,
}

impl ResearchRequest {
    pub fn new(topic: impl Into<String>, config: &ResearchConfig) -> Self {
        Self {
            topic: topic.into(),
            provider: config.provider.clone(),
            depth: config.depth.clone(),
            max_sources: config.max_sources,
            async_mode: config.async_mode,
            model: config.model.clone(),
            thinking_level: config.thinking_level.clone(),
        }
    }
}

/// Result from a research operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchResult {
    /// original research prompt/topic
    pub topic: String,
    /// Comprehensive summary/answer
    pub content: String,
    /// Sources used for the research
    #[serde(default)]
    pub sources: Vec<ResearchSource>,
    /// Provider used for the search
    pub provider: String,
}

/// A source used in the research
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResearchSource {
    /// The title of the web page or document
    pub title: String,
    /// The URL of the source
    pub url: String,
    /// A short snippet or excerpt from the content
    #[serde(default)]
    pub snippet: Option<String>,
}

/// Response from the research API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResearchResponse {
    /// Synchronous response with results
    Sync {
        /// Whether the operation was successful
        success: bool,
        /// The operation mode ("sync")
        mode: String,
        /// The actual research report or answer
        result: String,
        /// When the result was generated
        generated_at: String,
        /// Metadata about the search provider
        provider: String,
    },
    /// Asynchronous response with task ID
    Async {
        /// Whether the operation was successful
        success: bool,
        /// The operation mode ("async")
        mode: String,
        /// Unique identifier for the background task
        #[serde(rename = "taskId")]
        task_id: String,
        /// Informational message about task status
        message: String,
    },
}

// We need to be careful about the Sync response structure.
// In agents.ts:
// const result = await researchNetwork.run(researchPrompt);
// return c.json({ success: true, mode: "sync", result, ... });
//
// So 'result' is a string containing the markdown report.
//
// The SDK user probably wants a cleaner struct.
// Let's define a clean struct for success response.

/// Unified response structure for deep research operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepResearchResponse {
    /// Whether the operation was successfully initiated or completed
    pub success: bool,
    /// The operation mode ("sync" or "async")
    pub mode: String,
    /// The result of the research (only set for sync mode)
    pub result: Option<serde_json::Value>,
    /// The unique task identifier (only set for async mode)
    #[serde(rename = "taskId")]
    pub task_id: Option<String>,
    /// ISO 8601 timestamp of generation
    #[serde(rename = "generatedAt")]
    pub generated_at: Option<String>,
    /// The provider used for the operation
    pub provider: Option<String>,
    /// Informational message (e.g. error details or task status)
    pub message: Option<String>,
}

pub use crate::search::DeepResearchResponse as ResearchApiResponse;
