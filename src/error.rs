use reqwest::header::InvalidHeaderValue;
use reqwest::StatusCode;
use serde_json::Error as SerdeError;

#[derive(Debug, thiserror::Error)]
pub enum RainyError {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Invalid HTTP header value: {0}")]
    InvalidHeader(#[from] InvalidHeaderValue),

    #[error("JSON serialization/deserialization failed: {0}")]
    Json(#[from] SerdeError),

    #[error("Authentication failed: {message}")]
    Authentication { message: String },

    #[error("Authorization failed: {message}")]
    Authorization { message: String },

    #[error("Rate limit exceeded: {message}")]
    RateLimit { message: String },

    #[error("API error ({status}): {message}")]
    Api {
        status: StatusCode,
        message: String,
        code: Option<String>,
    },

    #[error("Validation error: {field} - {message}")]
    Validation { field: String, message: String },

    #[error("Network error: {0}")]
    Network(String),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Internal SDK error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, RainyError>;

impl RainyError {
    pub fn is_retryable(&self) -> bool {
        match self {
            RainyError::Http(err) => {
                err.is_timeout() || err.is_connect() || err.status().is_some_and(|s| s.is_server_error())
            },
            RainyError::RateLimit { .. } => true,
            RainyError::Network(_) => true,
            _ => false,
        }
    }

    pub fn should_retry(&self) -> bool {
        self.is_retryable()
    }
}