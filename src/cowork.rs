//! Cowork Integration Module
//!
//! This module provides types for Rainy Cowork integration.
//! All business logic (pricing, limits, features) comes from the API.

use serde::{Deserialize, Serialize};

/// Subscription plan details from the API
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CoworkPlan {
    /// Plan identifier (e.g., "plus", "pro")
    pub id: String,
    /// Human-readable name (e.g., "Plus", "Pro")
    pub name: String,
    /// Requests per minute limit
    #[serde(rename = "requestsPerMinute")]
    pub requests_per_minute: u32,
    /// Monthly usage limit
    #[serde(rename = "monthlyLimit")]
    pub monthly_limit: u32,
}

impl Default for CoworkPlan {
    fn default() -> Self {
        Self {
            id: "free".to_string(),
            name: "Free".to_string(),
            requests_per_minute: 5,
            monthly_limit: 30,
        }
    }
}

impl CoworkPlan {
    /// Check if this plan requires payment
    pub fn is_paid(&self) -> bool {
        self.id != "free"
    }
}

/// Feature flags for Cowork capabilities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoworkFeatures {
    /// Web research feature
    #[serde(default, rename = "web_research")]
    pub web_research: bool,
    /// Document export feature
    #[serde(default, rename = "document_export")]
    pub document_export: bool,
    /// Image analysis feature
    #[serde(default, rename = "image_analysis")]
    pub image_analysis: bool,
    /// Priority support feature
    #[serde(default, rename = "priority_support")]
    pub priority_support: bool,
}

/// Usage tracking for the current billing period
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoworkUsage {
    /// Requests used in current period
    #[serde(default)]
    pub used: u32,
    /// Maximum requests allowed
    #[serde(default)]
    pub limit: u32,
    /// Credits consumed
    #[serde(default, rename = "creditsUsed")]
    pub credits_used: f32,
}

impl CoworkUsage {
    /// Check if monthly usage limit is reached
    pub fn is_limit_reached(&self) -> bool {
        self.used >= self.limit
    }

    /// Get remaining uses
    pub fn remaining(&self) -> u32 {
        self.limit.saturating_sub(self.used)
    }
}

/// Profile response from /cowork/profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoworkProfile {
    /// User display name
    pub name: String,
    /// User email
    pub email: String,
    /// Subscription plan details
    pub plan: CoworkPlan,
    /// Current usage statistics
    pub usage: CoworkUsage,
}

/// Response from /cowork/models endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoworkModelsResponse {
    /// Plan identifier
    pub plan: String,
    /// Human-readable plan name
    pub plan_name: String,
    /// Access level (e.g. "basic", "standard", "high")
    pub model_access_level: String,
    /// List of available model IDs
    pub models: Vec<String>,
    /// Total number of available models
    pub total_models: u32,
}

/// Complete capabilities including features (constructed client-side or extended API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoworkCapabilities {
    pub profile: CoworkProfile,
    pub features: CoworkFeatures,
    pub is_valid: bool,
    pub models: Vec<String>,
    pub upgrade_message: Option<String>,
}

impl Default for CoworkCapabilities {
    fn default() -> Self {
        Self {
            profile: CoworkProfile {
                name: String::new(),
                email: String::new(),
                plan: CoworkPlan::default(),
                usage: CoworkUsage::default(),
            },
            features: CoworkFeatures::default(),
            is_valid: true,
            models: vec![],
            upgrade_message: None,
        }
    }
}

impl CoworkCapabilities {
    /// Create capabilities for free plan (offline/fallback)
    pub fn free() -> Self {
        Self::default()
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
            "priority_support" => self.features.priority_support,
            _ => false,
        }
    }

    /// Check if user can make more requests
    pub fn can_make_request(&self) -> bool {
        self.is_valid && !self.profile.usage.is_limit_reached()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_deserialize_profile() {
        let json = json!({
            "name": "Test User",
            "email": "test@example.com",
            "plan": {
                "id": "plus",
                "name": "Plus",
                "requestsPerMinute": 20,
                "monthlyLimit": 100
            },
            "usage": {
                "used": 15,
                "limit": 100,
                "creditsUsed": 2.5
            }
        });

        let profile: CoworkProfile = serde_json::from_value(json).unwrap();
        assert_eq!(profile.name, "Test User");
        assert_eq!(profile.plan.id, "plus");
        assert_eq!(profile.plan.requests_per_minute, 20);
        assert_eq!(profile.usage.credits_used, 2.5);
    }
}
