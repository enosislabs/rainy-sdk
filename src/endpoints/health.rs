use crate::client::RainyClient;
use crate::error::Result;
use crate::models::HealthCheck;

impl RainyClient {
    /// Checks the basic health of the Rainy API.
    ///
    /// This endpoint is useful for simple connectivity tests and monitoring.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HealthCheck` struct with basic status information.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::RainyClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("user-api-key")?;
    /// let health = client.health_check().await?;
    ///
    /// println!("API Status: {}", health.status);
    /// assert_eq!(health.status, "healthy");
    /// # Ok(())
    /// # }
    /// ```
    pub async fn health_check(&self) -> Result<HealthCheck> {
        self.make_request(reqwest::Method::GET, "/health", None)
            .await
    }

    /// Checks the detailed health of the API and its underlying services.
    ///
    /// This provides a more comprehensive health report, including the status of
    /// the database, Redis, and downstream AI providers.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HealthCheck` struct with detailed service information.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::RainyClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("user-api-key")?;
    /// let health = client.detailed_health_check().await?;
    ///
    /// println!("Database healthy: {}", health.services.database);
    /// println!("Providers healthy: {}", health.services.providers);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detailed_health_check(&self) -> Result<HealthCheck> {
        self.make_request(reqwest::Method::GET, "/health?detailed=true", None)
            .await
    }
}
