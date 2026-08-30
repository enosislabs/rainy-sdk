//! Public health operations.

use crate::client::RainyClient;
use crate::error::Result;
use crate::models::{HealthStatus, ServiceStatus};
use serde::Deserialize;
use serde_json::Value;

impl RainyClient {
    /// Performs the public root health check.
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

    /// Performs a detailed health check when the service exposes one.
    ///
    /// Only generic dependency labels are interpreted. Unknown fields are
    /// ignored so the SDK does not encode service-internal topology.
    pub async fn detailed_health_check(&self) -> Result<HealthStatus> {
        #[derive(Deserialize)]
        struct DependenciesHealthResponse {
            status: String,
            timestamp: String,
            #[serde(default)]
            dependencies: Value,
        }

        self.wait_for_slot().await;
        let response = self
            .send_request(self.root_request(reqwest::Method::GET, "/health/dependencies"))
            .await?;
        let payload: DependenciesHealthResponse = self.handle_response(response).await?;
        let dependencies = payload.dependencies.as_object();
        let bool_value = |names: &[&str]| {
            names
                .iter()
                .find_map(|name| dependencies.and_then(|items| items.get(*name)))
                .and_then(Value::as_bool)
                .unwrap_or(false)
        };
        Ok(HealthStatus {
            status: payload.status,
            timestamp: payload.timestamp,
            uptime: 0.0,
            services: ServiceStatus {
                database: bool_value(&["database"]),
                redis: dependencies
                    .and_then(|items| items.get("redis"))
                    .and_then(Value::as_bool),
                providers: bool_value(&["providers", "inference"]),
            },
        })
    }
}
