//! Cowork endpoint for retrieving subscription capabilities
//!
//! This endpoint validates the API key and returns the user's
//! Cowork plan, available models, and feature access.

use crate::{
    cowork::{CoworkCapabilities, CoworkFeatures, CoworkPlan, CoworkUsage},
    error::Result,
    RainyClient,
};
use serde::Deserialize;

/// Response from the Cowork capabilities API
#[derive(Debug, Clone, Deserialize)]
struct CoworkCapabilitiesResponse {
    plan: CoworkPlan,
    plan_name: String,
    is_valid: bool,
    usage: CoworkUsage,
    models: Vec<String>,
    features: CoworkFeatures,
    #[serde(default)]
    upgrade_message: Option<String>,
}

impl RainyClient {
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
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::RainyClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("your-api-key")?;
    /// let caps = client.get_cowork_capabilities().await?;
    ///
    /// if caps.plan.is_paid() {
    ///     println!("Paid plan with {} models available", caps.models.len());
    /// } else {
    ///     println!("Free plan - upgrade for more features");
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_cowork_capabilities(&self) -> Result<CoworkCapabilities> {
        let url = format!("{}/api/v1/cowork/capabilities", self.auth_config().base_url);

        let response = self.http_client().get(&url).send().await;

        match response {
            Ok(resp) => {
                if resp.status().is_success() {
                    let caps_response: CoworkCapabilitiesResponse =
                        self.handle_response(resp).await?;

                    Ok(CoworkCapabilities {
                        plan: caps_response.plan,
                        plan_name: caps_response.plan_name,
                        is_valid: caps_response.is_valid,
                        usage: caps_response.usage,
                        models: caps_response.models,
                        features: caps_response.features,
                        upgrade_message: caps_response.upgrade_message,
                    })
                } else {
                    // Invalid or expired API key - return free plan
                    Ok(CoworkCapabilities::free())
                }
            }
            Err(_) => {
                // Network error or API unavailable - return free plan
                Ok(CoworkCapabilities::free())
            }
        }
    }

    /// Check if the current API key grants paid access.
    ///
    /// This is a convenience method that calls `get_cowork_capabilities()`
    /// and checks if the plan is not Free.
    ///
    /// # Returns
    ///
    /// `true` if the user has a paid plan (GoPlus, Plus, Pro, or ProPlus).
    pub async fn has_paid_plan(&self) -> bool {
        match self.get_cowork_capabilities().await {
            Ok(caps) => caps.plan.is_paid(),
            Err(_) => false,
        }
    }

    /// Get available models for Cowork based on subscription plan.
    ///
    /// # Returns
    ///
    /// A vector of model identifiers available for the current plan.
    pub async fn get_cowork_models(&self) -> Result<Vec<String>> {
        let caps = self.get_cowork_capabilities().await?;
        Ok(caps.models)
    }

    /// Check if a specific feature is available.
    ///
    /// # Arguments
    ///
    /// * `feature` - Feature name: "web_research", "document_export", "image_analysis", etc.
    ///
    /// # Returns
    ///
    /// `true` if the feature is available for the current plan.
    pub async fn can_use_feature(&self, feature: &str) -> bool {
        match self.get_cowork_capabilities().await {
            Ok(caps) => caps.can_use_feature(feature),
            Err(_) => false,
        }
    }

    /// Check if a specific model is available for the current plan.
    ///
    /// # Arguments
    ///
    /// * `model` - The model identifier to check.
    ///
    /// # Returns
    ///
    /// `true` if the model is available for the current plan.
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
///
/// This can be used when network is unavailable to provide
/// a degraded experience based on previously cached plan info.
pub fn get_offline_capabilities(cached_plan: Option<CoworkPlan>) -> CoworkCapabilities {
    match cached_plan {
        Some(CoworkPlan::ProPlus)
        | Some(CoworkPlan::Pro)
        | Some(CoworkPlan::Plus)
        | Some(CoworkPlan::GoPlus) => {
            // For paid plans, we can't know exact capabilities offline
            // Return a minimal valid state
            CoworkCapabilities {
                plan: cached_plan.unwrap(),
                plan_name: cached_plan.unwrap().display_name().to_string(),
                is_valid: true,
                usage: CoworkUsage::default(),
                models: vec!["gemini-2.5-flash".to_string()],
                features: CoworkFeatures::default(),
                upgrade_message: None,
            }
        }
        _ => CoworkCapabilities::free(),
    }
}
