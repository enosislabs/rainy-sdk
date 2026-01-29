//! Web Research Module
//!
//! This module provides types and functionality for web research via Rainy API v2.
//! Supports multiple providers (Exa, Tavily) and configurable search depth.

use crate::models::{ResearchDepth, ResearchProvider};
use serde::{Deserialize, Serialize};

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

    /// Enable async mode
    pub fn with_async(mut self, async_mode: bool) -> Self {
        self.async_mode = async_mode;
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
}

impl ResearchRequest {
    pub fn new(topic: impl Into<String>, config: &ResearchConfig) -> Self {
        Self {
            topic: topic.into(),
            provider: config.provider.clone(),
            depth: config.depth.clone(),
            max_sources: config.max_sources,
            async_mode: config.async_mode,
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
    pub title: String,
    pub url: String,
    #[serde(default)]
    pub snippet: Option<String>,
}

/// Response from the research API
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResearchResponse {
    /// Synchronous response with results
    Sync {
        success: bool,
        mode: String,
        result: String, // The actual content answer (API returns 'result' as string in some paths, check agents.ts)
        // Wait, looking at agents.ts:
        // return c.json({ success: true, mode: "sync", result, generatedAt: ..., provider: ... });
        // result is the output from researchNetwork.run(prompt), which is a String.
        // But wait, does the network return just a string or a structured object?
        // researchNetwork.run returns a string usually (agent output).
        generated_at: String,
        provider: String,
    },
    /// Asynchronous response with task ID
    Async {
        success: bool,
        mode: String,
        #[serde(rename = "taskId")]
        task_id: String,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeepResearchResponse {
    pub success: bool,
    pub mode: String,
    pub result: Option<String>, // For sync
    #[serde(rename = "taskId")]
    pub task_id: Option<String>, // For async
    #[serde(rename = "generatedAt")]
    pub generated_at: Option<String>,
    pub provider: Option<String>,
    pub message: Option<String>,
}

pub use crate::search::DeepResearchResponse as ResearchApiResponse;
