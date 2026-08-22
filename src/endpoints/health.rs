use crate::client::RainyClient;
use crate::error::Result;
use crate::models::{HealthStatus, ServiceStatus};
use serde::Deserialize;

impl RainyClient {
    /// Performs a basic health check on the Rainy API.
    ///
    /// This method is useful for quickly verifying that the API is up and running.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HealthStatus` struct with basic health information.
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
    pub async fn health_check(&self) -> Result<HealthStatus> {
        #[derive(Deserialize)]
        struct RootHealthResponse {
            status: String,
            timestamp: String,
        }

        self.wait_for_slot().await;
        let response = self
            .send_request(self.root_request(reqwest::Method::GET, "/health"))
            .await?;
        let payload: RootHealthResponse = self.handle_response(response).await?;

        Ok(HealthStatus {
            status: payload.status,
            timestamp: payload.timestamp,
            uptime: 0.0,
            services: ServiceStatus {
                database: false,
                redis: None,
                providers: false,
            },
        })
    }

    /// Performs a detailed health check on the Rainy API and its underlying services.
    ///
    /// This method provides more in-depth information, including the status of the database,
    /// Redis, and connections to AI providers.
    ///
    /// # Returns
    ///
    /// A `Result` containing a `HealthStatus` struct with detailed service status.
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
    pub async fn detailed_health_check(&self) -> Result<HealthStatus> {
        #[derive(Deserialize)]
        struct DependencyFlags {
            database: bool,
            redis: bool,
            #[serde(rename = "openrouterConfigured")]
            openrouter_configured: bool,
            #[serde(rename = "polarConfigured")]
            polar_configured: bool,
        }
        #[derive(Deserialize)]
        struct DependenciesHealthResponse {
            status: String,
            timestamp: String,
            dependencies: DependencyFlags,
        }

        self.wait_for_slot().await;
        let response = self
            .send_request(self.root_request(reqwest::Method::GET, "/health/dependencies"))
            .await?;
        let payload: DependenciesHealthResponse = self.handle_response(response).await?;

        Ok(HealthStatus {
            status: payload.status,
            timestamp: payload.timestamp,
            uptime: 0.0,
            services: ServiceStatus {
                database: payload.dependencies.database,
                redis: Some(payload.dependencies.redis),
                providers: payload.dependencies.openrouter_configured
                    && payload.dependencies.polar_configured,
            },
        })
    }
}
