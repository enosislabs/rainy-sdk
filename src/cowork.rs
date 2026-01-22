//! Cowork Integration Module
//!
//! This module provides types for Rainy Cowork integration.
//! All business logic (pricing, limits, features) comes from the API.

use serde::{Deserialize, Serialize};

/// Subscription plan for Rainy Cowork
///
/// The actual plan details (pricing, limits) are returned by the API.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoworkPlan {
    /// Free tier - no payment required, limited access
    #[default]
    Free,
    /// Go Plus tier - entry-level paid subscription
    GoPlus,
    /// Plus tier - mid-level subscription with more features
    Plus,
    /// Pro tier - professional subscription with premium features
    Pro,
    /// Pro Plus tier - highest tier with all features
    ProPlus,
}

impl CoworkPlan {
    /// Check if this plan requires payment
    pub fn is_paid(&self) -> bool {
        !matches!(self, CoworkPlan::Free)
    }

    /// Get user-friendly display name
    pub fn display_name(&self) -> &'static str {
        match self {
            CoworkPlan::Free => "Free",
            CoworkPlan::GoPlus => "Go+",
            CoworkPlan::Plus => "Plus",
            CoworkPlan::Pro => "Pro",
            CoworkPlan::ProPlus => "Pro Plus",
        }
    }
}

/// Feature flags for Cowork capabilities (returned by API)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoworkFeatures {
    /// Web research feature - allows AI to browse and search the web
    #[serde(default)]
    pub web_research: bool,
    /// Document export feature - allows exporting to PDF/DOCX
    #[serde(default)]
    pub document_export: bool,
    /// Image analysis feature - allows AI vision capabilities
    #[serde(default)]
    pub image_analysis: bool,
    /// Priority support feature - faster response times
    #[serde(default)]
    pub priority_support: bool,
}

/// Usage tracking for the current billing period (returned by API)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CoworkUsage {
    /// The type of usage tracking (e.g., "monthly", "daily")
    #[serde(rename = "type", default)]
    pub usage_type: String,
    /// Number of requests/operations used in current period
    #[serde(default)]
    pub used: u32,
    /// Maximum number of requests/operations allowed in current period
    #[serde(default)]
    pub limit: u32,
    /// Credits consumed in current billing period
    #[serde(default)]
    pub credits_used: f32,
    /// Maximum credits allowed before rate limiting
    #[serde(default)]
    pub credits_ceiling: f32,
    /// ISO 8601 timestamp when usage counters reset
    #[serde(default)]
    pub resets_at: String,
}

impl CoworkUsage {
    /// Check if monthly usage limit is reached
    pub fn is_limit_reached(&self) -> bool {
        self.used >= self.limit
    }

    /// Check if credit ceiling is reached
    pub fn is_over_budget(&self) -> bool {
        self.credits_ceiling > 0.0 && self.credits_used >= self.credits_ceiling
    }

    /// Get remaining uses
    pub fn remaining(&self) -> u32 {
        self.limit.saturating_sub(self.used)
    }
}

/// Complete capabilities for a Cowork session (returned by API)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoworkCapabilities {
    /// The current subscription plan
    #[serde(default)]
    pub plan: CoworkPlan,
    /// Human-readable plan name
    #[serde(default)]
    pub plan_name: String,
    /// Whether the subscription is valid and active
    #[serde(default)]
    pub is_valid: bool,
    /// Current usage statistics for the billing period
    #[serde(default)]
    pub usage: CoworkUsage,
    /// List of AI model identifiers available for this plan
    #[serde(default)]
    pub models: Vec<String>,
    /// Feature flags indicating available capabilities
    #[serde(default)]
    pub features: CoworkFeatures,
    /// Optional message suggesting plan upgrade
    #[serde(skip_serializing_if = "Option::is_none")]
    pub upgrade_message: Option<String>,
}

impl Default for CoworkCapabilities {
    fn default() -> Self {
        Self::free()
    }
}

impl CoworkCapabilities {
    /// Create capabilities for free plan (offline/fallback)
    pub fn free() -> Self {
        Self {
            plan: CoworkPlan::Free,
            plan_name: "Free".to_string(),
            is_valid: true,
            usage: CoworkUsage::default(),
            models: vec![],
            features: CoworkFeatures::default(),
            upgrade_message: None,
        }
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
        self.is_valid && !self.usage.is_limit_reached() && !self.usage.is_over_budget()
    }
}

/// Backward compatibility alias
#[deprecated(since = "0.4.2", note = "Use CoworkPlan instead")]
pub type CoworkTier = CoworkPlan;

/// Backward compatibility alias
#[deprecated(since = "0.4.2", note = "Use CoworkUsage instead")]
pub type CoworkLimits = CoworkUsage;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plan_is_paid() {
        assert!(!CoworkPlan::Free.is_paid());
        assert!(CoworkPlan::GoPlus.is_paid());
        assert!(CoworkPlan::Plus.is_paid());
        assert!(CoworkPlan::Pro.is_paid());
        assert!(CoworkPlan::ProPlus.is_paid());
    }

    #[test]
    fn test_capabilities_free() {
        let caps = CoworkCapabilities::free();
        assert_eq!(caps.plan, CoworkPlan::Free);
        assert!(caps.models.is_empty());
    }

    #[test]
    fn test_usage_limits() {
        let usage = CoworkUsage {
            usage_type: "monthly".to_string(),
            used: 30,
            limit: 30,
            credits_used: 0.0,
            credits_ceiling: 0.0,
            resets_at: String::new(),
        };
        assert!(usage.is_limit_reached());
        assert_eq!(usage.remaining(), 0);
    }
}
