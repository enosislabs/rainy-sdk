//! Cowork Integration Module
//!
//! This module provides types and functionality for Rainy Cowork integration,
//! including subscription tiers, capabilities, and feature gating.
//!
//! # Architecture
//!
//! The Cowork module acts as a gatekeeper for premium features:
//! - Validates API keys and determines subscription tier
//! - Controls access to models based on tier
//! - Gates premium features (web research, document export, etc.)
//!
//! # Example
//!
//! ```rust,no_run
//! use rainy_sdk::{RainyClient, cowork::CoworkCapabilities};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let client = RainyClient::with_api_key("your-api-key")?;
//!     let caps = client.get_cowork_capabilities().await?;
//!     
//!     println!("Tier: {:?}", caps.tier);
//!     println!("Available models: {:?}", caps.models);
//!     println!("Web research: {}", caps.features.web_research);
//!     
//!     Ok(())
//! }
//! ```

use serde::{Deserialize, Serialize};

/// Subscription tier for Rainy Cowork
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CoworkTier {
    /// Free tier - no API key or invalid/expired key
    /// Users must provide their own Gemini API key
    #[default]
    Free,

    /// Basic tier - valid Rainy API key
    /// Access to standard models with usage limits
    Basic,

    /// Pro tier - paid subscription
    /// Full access to all models and features
    Pro,

    /// Enterprise tier - custom enterprise agreement
    /// Unlimited access, priority support, custom integrations
    Enterprise,
}

impl CoworkTier {
    /// Check if this tier has premium access
    pub fn is_premium(&self) -> bool {
        !matches!(self, CoworkTier::Free)
    }

    /// Check if this tier has full model access
    pub fn has_full_model_access(&self) -> bool {
        matches!(self, CoworkTier::Pro | CoworkTier::Enterprise)
    }
}

/// Feature flags for Cowork capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoworkFeatures {
    /// Can use web browsing and research features
    pub web_research: bool,

    /// Can export documents (PDF, DOCX)
    pub document_export: bool,

    /// Can use AI image analysis
    pub image_analysis: bool,

    /// Can use advanced automation workflows
    pub automation: bool,

    /// Can use priority queue for faster processing
    pub priority_queue: bool,

    /// Can access beta features
    pub beta_features: bool,
}

impl CoworkFeatures {
    /// Create features for free tier
    pub fn free() -> Self {
        Self::default()
    }

    /// Create features for basic tier
    pub fn basic() -> Self {
        Self {
            web_research: false,
            document_export: false,
            image_analysis: false,
            automation: false,
            priority_queue: false,
            beta_features: false,
        }
    }

    /// Create features for pro tier
    pub fn pro() -> Self {
        Self {
            web_research: true,
            document_export: true,
            image_analysis: true,
            automation: true,
            priority_queue: true,
            beta_features: false,
        }
    }

    /// Create features for enterprise tier
    pub fn enterprise() -> Self {
        Self {
            web_research: true,
            document_export: true,
            image_analysis: true,
            automation: true,
            priority_queue: true,
            beta_features: true,
        }
    }
}

/// Usage limits for the current billing period
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoworkLimits {
    /// Maximum tasks per day (None = unlimited)
    pub max_tasks_per_day: Option<u32>,

    /// Tasks used today
    pub tasks_used_today: u32,

    /// Maximum tokens per request (None = unlimited)
    pub max_tokens_per_request: Option<u32>,

    /// Maximum file size for processing in bytes
    pub max_file_size_bytes: Option<u64>,
}

impl CoworkLimits {
    /// Check if daily task limit is reached
    pub fn is_task_limit_reached(&self) -> bool {
        match self.max_tasks_per_day {
            Some(max) => self.tasks_used_today >= max,
            None => false,
        }
    }

    /// Get remaining tasks for today
    pub fn remaining_tasks(&self) -> Option<u32> {
        self.max_tasks_per_day
            .map(|max| max.saturating_sub(self.tasks_used_today))
    }
}

/// Complete capabilities for a Cowork session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoworkCapabilities {
    /// The subscription tier
    pub tier: CoworkTier,

    /// Available AI models for this tier
    pub models: Vec<String>,

    /// Feature flags
    pub features: CoworkFeatures,

    /// Usage limits
    pub limits: CoworkLimits,

    /// Whether the API key is valid
    pub is_valid: bool,

    /// Expiration date for the subscription (if applicable)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,

    /// Human-readable tier name
    pub tier_name: String,
}

impl Default for CoworkCapabilities {
    fn default() -> Self {
        Self::free()
    }
}

impl CoworkCapabilities {
    /// Create capabilities for free tier (no API key)
    pub fn free() -> Self {
        Self {
            tier: CoworkTier::Free,
            models: vec![],
            features: CoworkFeatures::free(),
            limits: CoworkLimits {
                max_tasks_per_day: Some(5),
                tasks_used_today: 0,
                max_tokens_per_request: Some(4096),
                max_file_size_bytes: Some(1024 * 1024), // 1MB
            },
            is_valid: false,
            expires_at: None,
            tier_name: "Free".to_string(),
        }
    }

    /// Create capabilities for basic tier
    pub fn basic() -> Self {
        use crate::models::model_constants;

        Self {
            tier: CoworkTier::Basic,
            models: vec![
                model_constants::OPENAI_GPT_4O.to_string(),
                model_constants::GOOGLE_GEMINI_2_5_FLASH.to_string(),
                model_constants::GOOGLE_GEMINI_2_5_FLASH_LITE.to_string(),
                model_constants::GROQ_LLAMA_3_1_8B_INSTANT.to_string(),
            ],
            features: CoworkFeatures::basic(),
            limits: CoworkLimits {
                max_tasks_per_day: Some(50),
                tasks_used_today: 0,
                max_tokens_per_request: Some(16384),
                max_file_size_bytes: Some(10 * 1024 * 1024), // 10MB
            },
            is_valid: true,
            expires_at: None,
            tier_name: "Basic".to_string(),
        }
    }

    /// Create capabilities for pro tier
    pub fn pro() -> Self {
        use crate::models::model_constants;

        Self {
            tier: CoworkTier::Pro,
            models: vec![
                // OpenAI
                model_constants::OPENAI_GPT_4O.to_string(),
                model_constants::OPENAI_GPT_5.to_string(),
                model_constants::OPENAI_GPT_5_PRO.to_string(),
                model_constants::OPENAI_O3.to_string(),
                model_constants::OPENAI_O4_MINI.to_string(),
                // Google
                model_constants::GOOGLE_GEMINI_2_5_PRO.to_string(),
                model_constants::GOOGLE_GEMINI_2_5_FLASH.to_string(),
                model_constants::GOOGLE_GEMINI_2_5_FLASH_LITE.to_string(),
                // Groq
                model_constants::GROQ_LLAMA_3_1_8B_INSTANT.to_string(),
                model_constants::GROQ_LLAMA_3_3_70B_VERSATILE.to_string(),
                // Cerebras
                model_constants::CEREBRAS_LLAMA3_1_8B.to_string(),
                // Enosis
                model_constants::ASTRONOMER_2.to_string(),
                model_constants::ASTRONOMER_2_PRO.to_string(),
            ],
            features: CoworkFeatures::pro(),
            limits: CoworkLimits {
                max_tasks_per_day: None, // Unlimited
                tasks_used_today: 0,
                max_tokens_per_request: None, // Unlimited
                max_file_size_bytes: Some(100 * 1024 * 1024), // 100MB
            },
            is_valid: true,
            expires_at: None,
            tier_name: "Pro".to_string(),
        }
    }

    /// Create capabilities for enterprise tier
    pub fn enterprise() -> Self {
        let mut caps = Self::pro();
        caps.tier = CoworkTier::Enterprise;
        caps.features = CoworkFeatures::enterprise();
        caps.limits.max_file_size_bytes = None; // Unlimited
        caps.tier_name = "Enterprise".to_string();
        caps
    }

    /// Check if a specific model is available
    pub fn can_use_model(&self, model: &str) -> bool {
        self.models.iter().any(|m| m == model)
    }

    /// Check if a specific feature is available
    pub fn can_use_feature(&self, feature: &str) -> bool {
        match feature {
            "web_research" => self.features.web_research,
            "document_export" => self.features.document_export,
            "image_analysis" => self.features.image_analysis,
            "automation" => self.features.automation,
            "priority_queue" => self.features.priority_queue,
            "beta_features" => self.features.beta_features,
            _ => false,
        }
    }
}

/// Response from the Cowork capabilities API
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct CoworkCapabilitiesResponse {
    pub tier: CoworkTier,
    pub tier_name: String,
    pub models: Vec<String>,
    pub features: CoworkFeatures,
    pub limits: CoworkLimits,
    #[serde(default)]
    pub expires_at: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tier_is_premium() {
        assert!(!CoworkTier::Free.is_premium());
        assert!(CoworkTier::Basic.is_premium());
        assert!(CoworkTier::Pro.is_premium());
        assert!(CoworkTier::Enterprise.is_premium());
    }

    #[test]
    fn test_capabilities_free() {
        let caps = CoworkCapabilities::free();
        assert_eq!(caps.tier, CoworkTier::Free);
        assert!(caps.models.is_empty());
        assert!(!caps.features.web_research);
    }

    #[test]
    fn test_capabilities_pro() {
        let caps = CoworkCapabilities::pro();
        assert_eq!(caps.tier, CoworkTier::Pro);
        assert!(!caps.models.is_empty());
        assert!(caps.features.web_research);
        assert!(caps.features.document_export);
    }

    #[test]
    fn test_limits_reached() {
        let mut limits = CoworkLimits {
            max_tasks_per_day: Some(5),
            tasks_used_today: 5,
            max_tokens_per_request: None,
            max_file_size_bytes: None,
        };
        assert!(limits.is_task_limit_reached());

        limits.tasks_used_today = 4;
        assert!(!limits.is_task_limit_reached());
    }
}
