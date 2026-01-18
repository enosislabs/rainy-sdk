//! Cowork endpoint for retrieving subscription capabilities
//!
//! This endpoint validates the API key and returns the user's
//! Cowork tier, available models, and feature access.

use crate::{
    cowork::{CoworkCapabilities, CoworkCapabilitiesResponse, CoworkTier},
    error::Result,
    RainyClient,
};

impl RainyClient {
    /// Retrieve Cowork capabilities for the current API key.
    ///
    /// This method validates the API key and returns information about:
    /// - Subscription tier (Free, Basic, Pro, Enterprise)
    /// - Available AI models
    /// - Feature access (web research, document export, etc.)
    /// - Usage limits
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
    /// if caps.tier.is_premium() {
    ///     println!("Premium user with {} models available", caps.models.len());
    /// } else {
    ///     println!("Free tier - please upgrade for more features");
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
                        tier: caps_response.tier,
                        tier_name: caps_response.tier_name,
                        models: caps_response.models,
                        features: caps_response.features,
                        limits: caps_response.limits,
                        is_valid: true,
                        expires_at: caps_response.expires_at,
                    })
                } else {
                    // Invalid or expired API key - return free tier
                    Ok(CoworkCapabilities::free())
                }
            }
            Err(_) => {
                // Network error or API unavailable - return free tier
                Ok(CoworkCapabilities::free())
            }
        }
    }

    /// Check if the current API key grants premium access.
    ///
    /// This is a convenience method that calls `get_cowork_capabilities()`
    /// and checks if the tier is not Free.
    ///
    /// # Returns
    ///
    /// `true` if the user has premium (Basic, Pro, or Enterprise) access.
    pub async fn is_premium(&self) -> bool {
        match self.get_cowork_capabilities().await {
            Ok(caps) => caps.tier.is_premium(),
            Err(_) => false,
        }
    }

    /// Get available models for Cowork based on subscription tier.
    ///
    /// # Returns
    ///
    /// A vector of model identifiers available for the current tier.
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
    /// `true` if the feature is available for the current tier.
    pub async fn can_use_feature(&self, feature: &str) -> bool {
        match self.get_cowork_capabilities().await {
            Ok(caps) => caps.can_use_feature(feature),
            Err(_) => false,
        }
    }

    /// Check if a specific model is available for the current tier.
    ///
    /// # Arguments
    ///
    /// * `model` - The model identifier to check.
    ///
    /// # Returns
    ///
    /// `true` if the model is available for the current tier.
    pub async fn can_use_model(&self, model: &str) -> bool {
        match self.get_cowork_capabilities().await {
            Ok(caps) => caps.can_use_model(model),
            Err(_) => false,
        }
    }
}

/// Offline tier detection based on cached data.
///
/// This can be used when network is unavailable to provide
/// a degraded experience based on previously cached tier info.
pub fn get_offline_capabilities(cached_tier: Option<CoworkTier>) -> CoworkCapabilities {
    match cached_tier {
        Some(CoworkTier::Enterprise) => CoworkCapabilities::enterprise(),
        Some(CoworkTier::Pro) => CoworkCapabilities::pro(),
        Some(CoworkTier::Basic) => CoworkCapabilities::basic(),
        _ => CoworkCapabilities::free(),
    }
}
