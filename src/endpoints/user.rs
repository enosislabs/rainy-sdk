use crate::client::RainyClient;
use crate::error::Result;
use crate::models::{CreditInfo, User};

impl RainyClient {
    /// Fetches detailed information about the currently authenticated user's account.
    ///
    /// This includes the user's plan, credit balance, and account status.
    ///
    /// # Returns
    ///
    /// A `Result` containing the `User` object for the authenticated user.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::RainyClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("user-api-key")?;
    ///
    /// let user = client.get_user_account().await?;
    /// println!("User ID: {}", user.user_id);
    /// println!("Current credits: {:.2}", user.current_credits);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_user_account(&self) -> Result<User> {
        self.make_request(reqwest::Method::GET, "/users/account", None)
            .await
    }

    /// Fetches the current credit balance for the user's account.
    ///
    /// This is the recommended way to check the current credit balance.
    ///
    /// # Returns
    ///
    /// A `Result` containing `CreditInfo` with the user's current balance.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::RainyClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("user-api-key")?;
    ///
    /// let credits = client.get_credit_info().await?;
    /// println!("Current credit balance: {:.2}", credits.current_credits);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_credit_info(&self) -> Result<CreditInfo> {
        self.make_request(reqwest::Method::GET, "/usage/credits", None)
            .await
    }
}
