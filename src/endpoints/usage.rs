use crate::client::RainyClient;
use crate::error::Result;
use crate::models::{CreditInfo, UsageStats};

impl RainyClient {
    /// Fetches information about the user's current credit balance and estimated costs.
    ///
    /// This endpoint is deprecated in favor of `get_credit_info`.
    ///
    /// # Arguments
    ///
    /// * `days` - Optional number of days of usage to consider for estimations. Defaults to 30.
    ///
    /// # Returns
    ///
    /// A `Result` containing `CreditInfo` with the user's current balance and cost estimates.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::RainyClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("user-api-key")?;
    ///
    /// let credits = client.get_credit_stats(Some(7)).await?;
    /// println!("Current credits: {}", credits.current_credits);
    /// # Ok(())
    /// # }
    /// ```
    #[deprecated(since = "0.2.0", note = "Please use `get_credit_info` instead")]
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

    /// Fetches detailed usage statistics for the user's account.
    ///
    /// This includes total requests, total tokens, a daily breakdown of usage,
    /// and a list of recent credit transactions.
    ///
    /// # Arguments
    ///
    /// * `days` - The number of past days to include in the statistics. Defaults to 30.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `UsageStats` object with detailed usage data.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::RainyClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("user-api-key")?;
    ///
    /// let usage = client.get_usage_stats(Some(30)).await?;
    /// println!("Total requests in the last 30 days: {}", usage.total_requests);
    /// println!("Total tokens in the last 30 days: {}", usage.total_tokens);
    ///
    /// if let Some(daily) = usage.daily_usage.first() {
    ///     println!("{}: {:.2} credits used", daily.date, daily.credits_used);
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
