use crate::client::RainyClient;
use crate::error::Result;
use crate::models::{CreditInfo, UsageStats};

impl RainyClient {
    /// Get credit statistics
    ///
    /// This endpoint returns information about the user's credit usage.
    ///
    /// # Arguments
    ///
    /// * `days` - Optional number of days to look back (default: 30)
    ///
    /// # Returns
    ///
    /// Returns credit information including balance, usage, and allocation.
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
    /// let credits = client.get_credit_stats(Some(7)).await?;
    /// println!("Current balance: {}", credits.current_balance);
    /// println!("Used this month: {}", credits.used_this_month);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_credit_stats(&self, days: Option<u32>) -> Result<CreditInfo> {
        let endpoint = if let Some(days) = days {
            format!("/usage/credits?days={days}")
        } else {
            "/usage/credits".to_string()
        };

        #[derive(serde::Deserialize)]
        struct CreditStatsResponse {
            credits: CreditInfo,
        }

        let response: CreditStatsResponse = self
            .make_request(reqwest::Method::GET, &endpoint, None)
            .await?;

        Ok(response.credits)
    }

    /// Get usage statistics
    ///
    /// This endpoint returns detailed usage statistics.
    ///
    /// # Arguments
    ///
    /// * `days` - Optional number of days to look back (default: 30)
    ///
    /// # Returns
    ///
    /// Returns comprehensive usage statistics including daily usage and recent transactions.
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
    /// let usage = client.get_usage_stats(Some(30)).await?;
    /// println!("Total requests: {}", usage.total_requests);
    /// println!("Total tokens: {}", usage.total_tokens);
    ///
    /// for daily in &usage.daily_usage {
    ///     println!("{}: {} credits used", daily.date, daily.credits_used);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_usage_stats(&self, days: Option<u32>) -> Result<UsageStats> {
        let endpoint = if let Some(days) = days {
            format!("/usage/stats?days={days}")
        } else {
            "/usage/stats".to_string()
        };

        self.make_request(reqwest::Method::GET, &endpoint, None)
            .await
    }
}
