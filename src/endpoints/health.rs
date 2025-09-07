use crate::client::RainyClient;
use crate::error::Result;
use crate::models::HealthCheck;

impl RainyClient {
    /// Performs a basic health check on the Rainy API.
    ///
    /// This method is useful for quickly verifying that the API is up and running.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HealthCheck` struct with basic health information.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::RainyClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("user-api-key")?;
    /// let health = client.health_check().await?;
    /// println!("API Status: {}", health.status);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn health_check(&self) -> Result<HealthCheck> {
        self.make_request(reqwest::Method::GET, "/health", None)
            .await
    }

    /// Performs a detailed health check on the Rainy API and its underlying services.
    ///
    /// This method provides more in-depth information, including the status of the database,
    /// Redis, and connections to AI providers.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HealthCheck` struct with detailed service status.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use rainy_sdk::RainyClient;
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let client = RainyClient::with_api_key("user-api-key")?;
    /// let health = client.detailed_health_check().await?;
    /// println!("Database status: {}", health.services.database);
    /// println!("Providers status: {}", health.services.providers);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detailed_health_check(&self) -> Result<HealthCheck> {
        self.make_request(reqwest::Method::GET, "/health?detailed=true", None)
            .await
    }
}
