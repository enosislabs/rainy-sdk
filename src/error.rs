//! Public error taxonomy with safe, non-secret diagnostics.

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Errors returned by the SDK.
#[derive(Error, Debug, Clone)]
pub enum RainyError {
    /// Authentication or credential validation failed.
    #[error("Authentication failed: {message}")]
    Authentication {
        /// Machine-readable code.
        code: String,
        /// Safe human-readable message.
        message: String,
        /// Whether a caller may retry this operation.
        retryable: bool,
    },

    /// The request could not be represented or accepted by the protocol.
    #[error("Invalid request: {message}")]
    InvalidRequest {
        /// Machine-readable code.
        code: String,
        /// Safe human-readable message.
        message: String,
        /// Structured validation details, when useful.
        details: Option<serde_json::Value>,
    },

    /// An upstream provider error exposed by a compatible gateway.
    #[error("Provider error ({provider}): {message}")]
    Provider {
        /// Provider-defined code.
        code: String,
        /// Safe provider message.
        message: String,
        /// Opaque provider label, if supplied by the API.
        provider: String,
        /// Whether the API marked it retryable.
        retryable: bool,
    },

    /// A rate limit response.
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        /// Machine-readable code.
        code: String,
        /// Safe human-readable message.
        message: String,
        /// Bounded server-provided retry delay.
        retry_after: Option<u64>,
        /// Optional usage summary.
        current_usage: Option<String>,
    },

    /// The service rejected a request for lack of credits.
    #[error("Insufficient credits: {message}")]
    InsufficientCredits {
        /// Machine-readable code.
        code: String,
        /// Safe human-readable message.
        message: String,
        /// Current balance.
        current_credits: f64,
        /// Estimated cost.
        estimated_cost: f64,
        /// Reset date when supplied.
        reset_date: Option<String>,
    },

    /// A network or connection error.
    #[error("Network error: {message}")]
    Network {
        /// Safe high-level message.
        message: String,
        /// Whether the operation may be retried if it is otherwise safe.
        retryable: bool,
        /// Optional deliberately sanitized source detail.
        source_error: Option<String>,
    },

    /// A non-specialized HTTP API error.
    #[error("API error [{status_code}]: {message}")]
    Api {
        /// Machine-readable code.
        code: String,
        /// Safe response message.
        message: String,
        /// HTTP status code.
        status_code: u16,
        /// Whether the status is transient.
        retryable: bool,
        /// Request correlation identifier.
        request_id: Option<String>,
    },

    /// A request timeout.
    #[error("Request timeout: {message}")]
    Timeout {
        /// Safe high-level message.
        message: String,
        /// Configured timeout in milliseconds.
        duration_ms: u64,
    },

    /// JSON serialization/deserialization failed.
    #[error("Serialization error: {message}")]
    Serialization {
        /// Safe parse/serialization message.
        message: String,
        /// Non-secret parser detail.
        source_error: Option<String>,
    },

    /// A feature is not available on the selected service.
    #[error("Feature not available: {feature} - {message}")]
    FeatureNotAvailable {
        /// Feature name.
        feature: String,
        /// Explanation.
        message: String,
    },

    /// The service denied a model or inference capability.
    #[error("Access denied ({code}): {message}")]
    AccessDenied {
        /// Machine-readable code.
        code: String,
        /// Safe explanation.
        message: String,
        /// Structured details, when supplied.
        details: Option<serde_json::Value>,
    },

    /// The request or response exceeded an SDK safety bound.
    #[error("Payload too large: {message} (maximum {max_bytes} bytes)")]
    PayloadTooLarge {
        /// Safe high-level message.
        message: String,
        /// Enforced limit.
        max_bytes: usize,
    },

    /// Compatibility alias for older callers.
    #[error("Network error: {0}")]
    NetworkError(String),

    /// Compatibility alias for older callers.
    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl RainyError {
    /// Returns whether this error is generally transient.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Authentication { retryable, .. }
            | Self::Provider { retryable, .. }
            | Self::Network { retryable, .. }
            | Self::Api { retryable, .. } => *retryable,
            Self::RateLimit { .. } | Self::Timeout { .. } => true,
            _ => false,
        }
    }

    /// Returns a bounded server-requested retry delay.
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            Self::RateLimit { retry_after, .. } => *retry_after,
            _ => None,
        }
    }

    /// Returns the machine-readable code, if one exists.
    pub fn code(&self) -> Option<&str> {
        match self {
            Self::Authentication { code, .. }
            | Self::InvalidRequest { code, .. }
            | Self::Provider { code, .. }
            | Self::RateLimit { code, .. }
            | Self::InsufficientCredits { code, .. }
            | Self::AccessDenied { code, .. }
            | Self::Api { code, .. } => Some(code),
            _ => None,
        }
    }

    /// Returns a request correlation identifier, if one exists.
    pub fn request_id(&self) -> Option<&str> {
        match self {
            Self::Api { request_id, .. } => request_id.as_deref(),
            _ => None,
        }
    }
}

/// Standard Rainy error response shape.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    /// Error details.
    pub error: ApiErrorDetails,
}

/// Detailed standard Rainy error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorDetails {
    /// Machine-readable error code.
    pub code: String,
    /// Human-readable message.
    pub message: String,
    /// Structured details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Server retry hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    /// Error timestamp.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// Request identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// Result alias used throughout the crate.
pub type Result<T> = std::result::Result<T, RainyError>;

impl From<reqwest::Error> for RainyError {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            Self::Timeout {
                message: "Request timed out".to_string(),
                duration_ms: 0,
            }
        } else {
            Self::Network {
                message: if error.is_connect() {
                    "Could not connect to the service".to_string()
                } else if error.is_request() {
                    "The HTTP request could not be sent".to_string()
                } else {
                    "The HTTP request failed".to_string()
                },
                retryable: error.is_connect() || error.is_request(),
                source_error: None,
            }
        }
    }
}

impl From<serde_json::Error> for RainyError {
    fn from(error: serde_json::Error) -> Self {
        Self::Serialization {
            message: error.to_string(),
            source_error: Some(error.to_string()),
        }
    }
}

impl From<reqwest::header::InvalidHeaderValue> for RainyError {
    fn from(_: reqwest::header::InvalidHeaderValue) -> Self {
        Self::InvalidRequest {
            code: "INVALID_HEADER".to_string(),
            message: "request contained an invalid header value".to_string(),
            details: None,
        }
    }
}
