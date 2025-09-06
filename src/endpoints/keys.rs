use crate::client::RainyClient;
use crate::error::Result;
use crate::models::ApiKey;

impl RainyClient {
    /// Creates a new API key.
    ///
    /// This endpoint allows for the programmatic creation of new API keys, which can
    /// be useful for provisioning users or services. A master API key with appropriate
    /// permissions is required.
    ///
    /// # Arguments
    ///
    /// * `description` - A human-readable description of what the key will be used for.
    /// * `expires_in_days` - An optional number of days until the key expires.
    ///
    /// # Returns
    ///
    /// A `Result` containing the newly created `ApiKey`. The actual key string is
    /// only returned on creation.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::RainyClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("master-api-key")?;
    ///
    /// let new_key = client.create_api_key(
    ///     "Temporary key for batch processing",
    ///     Some(7)
    /// ).await?;
    ///
    /// println!("Created API key: {}", new_key.key);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_api_key(
        &self,
        description: &str,
        expires_in_days: Option<u32>,
    ) -> Result<ApiKey> {
        let mut body = serde_json::json!({
            "description": description
        });

        if let Some(days) = expires_in_days {
            body["expiresInDays"] = serde_json::json!(days);
        }

        self.make_request(reqwest::Method::POST, "/keys", Some(body))
            .await
    }

    /// Lists all API keys associated with the current user account.
    ///
    /// # Returns
    ///
    /// A `Result` containing a vector of `ApiKey` structs. Note that the `key`
    /// field in the returned structs will be masked for security.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::RainyClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("user-api-key")?;
    ///
    /// let keys = client.list_api_keys().await?;
    /// for key in keys {
    ///     println!("Key ID: {} - Description: {}", key.id, key.description.as_deref().unwrap_or("N/A"));
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_api_keys(&self) -> Result<Vec<ApiKey>> {
        #[derive(serde::Deserialize)]
        struct ApiKeysResponse {
            api_keys: Vec<ApiKey>,
        }

        let response: ApiKeysResponse = self
            .make_request(reqwest::Method::GET, "/keys", None)
            .await?;

        Ok(response.api_keys)
    }

    /// Updates the description or status of an existing API key.
    ///
    /// # Arguments
    ///
    /// * `key_id` - The UUID of the API key to update.
    /// * `updates` - A `serde_json::Value` object containing the fields to update.
    ///   For example, `json!({ "description": "New description", "is_active": false })`.
    ///
    /// # Returns
    ///
    /// A `Result` containing the updated `ApiKey`.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::{RainyClient, AuthConfig};
    /// # use serde_json::json;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("user-api-key")?;
    /// let key_id_to_update = "550e8400-e29b-41d4-a716-446655440000";
    ///
    /// let updates = json!({
    ///     "description": "Updated key description"
    /// });
    ///
    /// let updated_key = client.update_api_key(key_id_to_update, updates).await?;
    /// assert_eq!(updated_key.description.as_deref(), Some("Updated key description"));
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

    /// Deletes an API key.
    ///
    /// This action is irreversible.
    ///
    /// # Arguments
    ///
    /// * `key_id` - The UUID of the API key to delete.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::RainyClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("user-api-key")?;
    /// let key_id_to_delete = "550e8400-e29b-41d4-a716-446655440000";
    ///
    /// client.delete_api_key(key_id_to_delete).await?;
    /// println!("API key deleted successfully");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_api_key(&self, key_id: &str) -> Result<()> {
        self.make_request(reqwest::Method::DELETE, &format!("/keys/{key_id}"), None)
            .await
    }
}
