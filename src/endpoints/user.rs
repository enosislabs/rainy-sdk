use crate::client::RainyClient;
use crate::error::Result;
use crate::models::User;

impl RainyClient {
    /// Get current user account information
    ///
    /// This endpoint requires user authentication with an API key.
    ///
    /// # Returns
    ///
    /// Returns information about the authenticated user.
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
    /// let user = client.get_user_account().await?;
    /// println!("Current credits: {}", user.current_credits);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_user_account(&self) -> Result<User> {
        self.make_request(reqwest::Method::GET, "/users/account", None)
            .await
    }
}
