use crate::client::RainyClient;
use crate::error::Result;
use crate::models::ApiKey;

impl RainyClient {
    /// Create a new API key
    ///
    /// This endpoint requires user authentication with a master API key.
    ///
    /// # Arguments
    ///
    /// * `description` - Description of what this API key will be used for
    /// * `expires_in_days` - Optional expiration time in days
    ///
    /// # Returns
    ///
    /// Returns the created API key information.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::{RainyClient, AuthConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::new(
    ///     AuthConfig::new().with_api_key("master-api-key")
    /// )?;
    ///
    /// let api_key = client.create_api_key(
    ///     "Production API key",
    ///     Some(365)
    /// ).await?;
    ///
    /// println!("Created API key: {}", api_key.key);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_api_key(&self, description: &str, expires_in_days: Option<u32>) -> Result<ApiKey> {
        let mut body = serde_json::json!({
            "description": description
        });

        if let Some(days) = expires_in_days {
            body["expiresInDays"] = serde_json::json!(days);
        }

        self.make_request(reqwest::Method::POST, "/keys", Some(body))
            .await
    }

    /// List all API keys for the current user
    ///
    /// This endpoint requires user authentication.
    ///
    /// # Returns
    ///
    /// Returns a vector of all API keys owned by the authenticated user.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::{RainyClient, AuthConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::new(
    ///     AuthConfig::new().with_api_key("user-api-key")
    /// )?;
    ///
    /// let keys = client.list_api_keys().await?;
    /// for key in keys {
    ///     println!("Key: {} - Active: {}", key.key, key.is_active);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_api_keys(&self) -> Result<Vec<ApiKey>> {
        #[derive(serde::Deserialize)]
        struct ApiKeysResponse {
            api_keys: Vec<ApiKey>,
        }

        let response: ApiKeysResponse = self.make_request(
            reqwest::Method::GET,
            "/users/account",
            None,
        )
        .await?;

        Ok(response.api_keys)
    }

    /// Update an API key
    ///
    /// This endpoint requires user authentication.
    ///
    /// # Arguments
    ///
    /// * `key_id` - The UUID of the API key to update
    /// * `updates` - JSON object containing the fields to update
    ///
    /// # Returns
    ///
    /// Returns the updated API key information.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::{RainyClient, AuthConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use serde_json::json;
    ///
    /// let client = RainyClient::new(
    ///     AuthConfig::new().with_api_key("user-api-key")
    /// )?;
    ///
    /// let updates = json!({
    ///     "description": "Updated description"
    /// });
    ///
    /// let updated_key = client.update_api_key(
    ///     "550e8400-e29b-41d4-a716-446655440000",
    ///     updates
    /// ).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_api_key(&self, key_id: &str, updates: serde_json::Value) -> Result<ApiKey> {
        self.make_request(
            reqwest::Method::PATCH,
            &format!("/keys/{key_id}"),
            Some(updates),
        )
        .await
    }

    /// Delete an API key
    ///
    /// This endpoint requires user authentication.
    ///
    /// # Arguments
    ///
    /// * `key_id` - The UUID of the API key to delete
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::{RainyClient, AuthConfig};
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::new(
    ///     AuthConfig::new().with_api_key("user-api-key")
    /// )?;
    ///
    /// client.delete_api_key("550e8400-e29b-41d4-a716-446655440000").await?;
    /// println!("API key deleted successfully");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_api_key(&self, key_id: &str) -> Result<()> {
        self.make_request(
            reqwest::Method::DELETE,
            &format!("/keys/{key_id}"),
            None,
        )
        .await
    }
}
