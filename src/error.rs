use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Comprehensive error type for Rainy SDK operations
#[derive(Error, Debug, Clone)]
pub enum RainyError {
    /// Authentication-related errors
    #[error("Authentication failed: {message}")]
    Authentication {
        code: String,
        message: String,
        retryable: bool,
    },

    /// Request validation errors
    #[error("Invalid request: {message}")]
    InvalidRequest {
        code: String,
        message: String,
        details: Option<serde_json::Value>,
    },

    /// Provider-specific errors
    #[error("Provider error ({provider}): {message}")]
    Provider {
        code: String,
        message: String,
        provider: String,
        retryable: bool,
    },

    /// Rate limiting errors
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        code: String,
        message: String,
        retry_after: Option<u64>,
        current_usage: Option<String>,
    },

    /// Credit system errors
    #[error("Insufficient credits: {message}")]
    InsufficientCredits {
        code: String,
        message: String,
        current_credits: f64,
        estimated_cost: f64,
        reset_date: Option<String>,
    },

    /// Network-related errors
    #[error("Network error: {message}")]
    Network {
        message: String,
        retryable: bool,
        source_error: Option<String>,
    },

    /// General API errors
    #[error("API error [{status_code}]: {message}")]
    Api {
        code: String,
        message: String,
        status_code: u16,
        retryable: bool,
        request_id: Option<String>,
    },

    /// Timeout errors
    #[error("Request timeout: {message}")]
    Timeout { message: String, duration_ms: u64 },

    /// Serialization/deserialization errors
    #[error("Serialization error: {message}")]
    Serialization {
        message: String,
        source_error: Option<String>,
    },
}

impl RainyError {
    /// Check if the error is retryable
    pub fn is_retryable(&self) -> bool {
        match self {
            RainyError::Authentication { retryable, .. } => *retryable,
            RainyError::Provider { retryable, .. } => *retryable,
            RainyError::Network { retryable, .. } => *retryable,
            RainyError::Api { retryable, .. } => *retryable,
            RainyError::RateLimit { .. } => true,
            RainyError::Timeout { .. } => true,
            _ => false,
        }
    }

    /// Get retry delay in seconds if applicable
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            RainyError::RateLimit { retry_after, .. } => *retry_after,
            _ => None,
        }
    }

    /// Get the error code
    pub fn code(&self) -> Option<&str> {
        match self {
            RainyError::Authentication { code, .. }
            | RainyError::InvalidRequest { code, .. }
            | RainyError::Provider { code, .. }
            | RainyError::RateLimit { code, .. }
            | RainyError::InsufficientCredits { code, .. }
            | RainyError::Api { code, .. } => Some(code),
            _ => None,
        }
    }

    /// Get request ID if available
    pub fn request_id(&self) -> Option<&str> {
        match self {
            RainyError::Api { request_id, .. } => request_id.as_deref(),
            _ => None,
        }
    }
}

/// API error response structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub error: ApiErrorDetails,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorDetails {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, RainyError>;

/// Convert reqwest errors to RainyError
impl From<reqwest::Error> for RainyError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            RainyError::Timeout {
                message: "Request timed out".to_string(),
                duration_ms: 30000, // Default timeout
            }
        } else if err.is_connect() || err.is_request() {
            RainyError::Network {
                message: err.to_string(),
                retryable: true,
                source_error: Some(err.to_string()),
            }
        } else {
            RainyError::Network {
                message: err.to_string(),
                retryable: false,
                source_error: Some(err.to_string()),
            }
        }
    }
}

/// Convert serde_json errors to RainyError
impl From<serde_json::Error> for RainyError {
    fn from(err: serde_json::Error) -> Self {
        RainyError::Serialization {
            message: err.to_string(),
            source_error: Some(err.to_string()),
        }
    }
}

/// Convert reqwest header errors to RainyError  
impl From<reqwest::header::InvalidHeaderValue> for RainyError {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        RainyError::InvalidRequest {
            code: "INVALID_HEADER".to_string(),
            message: format!("Invalid header value: {}", err),
            details: None,
        }
    }
}
