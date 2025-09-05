use crate::client::RainyClient;
use crate::error::Result;
use crate::models::HealthCheck;

impl RainyClient {
    /// Check the health of the API
    ///
    /// # Returns
    ///
    /// Returns basic health information about the API.
    pub async fn health_check(&self) -> Result<HealthCheck> {
        self.make_request(reqwest::Method::GET, "/health", None)
            .await
    }

    /// Check the detailed health of the API and its services
    ///
    /// # Returns
    ///
    /// Returns detailed health information, including service status.
    pub async fn detailed_health_check(&self) -> Result<HealthCheck> {
        self.make_request(reqwest::Method::GET, "/health?detailed=true", None)
            .await
    }
}
