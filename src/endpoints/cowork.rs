//! Cowork endpoint for retrieving subscription capabilities
//!
//! This endpoint validates the API key and returns the user's
//! Cowork plan, available models, and feature access.

use crate::{
    cowork::{
        CoworkCapabilities, CoworkFeatures, CoworkModelsResponse, CoworkPlan, CoworkProfile,
        CoworkUsage,
    },
    error::Result,
    RainyClient,
};

impl RainyClient {
    /// Retrieve available models for the current Cowork plan directly from the API.
    ///
    /// This is more efficient than fetching full capabilities if only models are needed.
    pub async fn get_cowork_models(&self) -> Result<CoworkModelsResponse> {
        self.make_request(reqwest::Method::GET, "/cowork/models", None)
            .await
    }

    /// Retrieve Cowork capabilities for the current API key.
    ///
    /// This method validates the API key and returns information about:
    /// - Subscription plan (Free, GoPlus, Plus, Pro, ProPlus)
    /// - Available AI models
    /// - Feature access (web research, document export, etc.)
    /// - Usage tracking
    ///
    /// # Returns
    ///
    /// A `Result` containing `CoworkCapabilities` on success, or a `RainyError` on failure.
    pub async fn get_cowork_capabilities(&self) -> Result<CoworkCapabilities> {
        match self.get_cowork_profile().await {
            Ok(profile) => {
                // In a real app, features/models might come from the API too,
                // or be derived from the plan ID.
                // For now, we simulate them based on plan ID or use defaults.
                // Assuming the API returns features/models in the profile response is better,
                // but if not, logic goes here.
                // The new CoworkProfile struct does NOT have models/features, so we map them here.

                let models = match profile.plan.id.as_str() {
                    "free" => vec!["gemini-2.0-flash".to_string()],
                    "go" | "go_plus" => {
                        vec![
                            "gemini-2.0-flash".to_string(),
                            "gemini-2.5-flash".to_string(),
                        ]
                    }
                    "plus" => vec![
                        "gemini-2.0-flash".to_string(),
                        "gemini-2.5-flash".to_string(),
                        "gemini-2.5-pro".to_string(),
                    ],
                    "pro" | "pro_plus" => vec![
                        "gemini-2.0-flash".to_string(),
                        "gemini-2.5-flash".to_string(),
                        "gemini-2.5-pro".to_string(),
                        "claude-sonnet-4".to_string(),
                    ],
                    _ => vec![],
                };

                let features = CoworkFeatures {
                    web_research: profile.plan.id != "free",
                    document_export: profile.plan.id != "free",
                    image_analysis: true,
                    priority_support: profile.plan.id.contains("pro"),
                };

                Ok(CoworkCapabilities {
                    profile,
                    features,
                    is_valid: true,
                    models,
                    upgrade_message: None,
                })
            }
            Err(_) => Ok(CoworkCapabilities::free()),
        }
    }

    /// Check if the current API key grants paid access.
    pub async fn has_paid_plan(&self) -> bool {
        match self.get_cowork_capabilities().await {
            Ok(caps) => caps.profile.plan.is_paid(),
            Err(_) => false,
        }
    }

    /// Check if a specific feature is available.
    pub async fn can_use_feature(&self, feature: &str) -> bool {
        match self.get_cowork_capabilities().await {
            Ok(caps) => caps.can_use_feature(feature),
            Err(_) => false,
        }
    }

    /// Check if a specific model is available for the current plan.
    pub async fn can_use_model(&self, model: &str) -> bool {
        match self.get_cowork_capabilities().await {
            Ok(caps) => caps.can_use_model(model),
            Err(_) => false,
        }
    }

    /// Check if user can make another request based on usage limits.
    pub async fn can_make_request(&self) -> bool {
        match self.get_cowork_capabilities().await {
            Ok(caps) => caps.can_make_request(),
            Err(_) => false,
        }
    }
}

/// Offline capabilities based on cached plan data.
pub fn get_offline_capabilities(cached_plan: Option<CoworkPlan>) -> CoworkCapabilities {
    match cached_plan {
        Some(plan) if plan.is_paid() => {
            // Reconstruct minimal capabilities
            CoworkCapabilities {
                profile: CoworkProfile {
                    name: "Offline User".to_string(),
                    email: "".to_string(),
                    plan,
                    usage: CoworkUsage::default(),
                },
                features: CoworkFeatures::default(), // Pessimistic offline features
                is_valid: true,
                models: vec!["gemini-2.5-flash".to_string()],
                upgrade_message: None,
            }
        }
        _ => CoworkCapabilities::free(),
    }
}
