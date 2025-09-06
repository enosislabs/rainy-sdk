use serde::{Deserialize, Serialize};
use thiserror::Error;

/// A comprehensive error type for all operations within the Rainy SDK.
///
/// This enum consolidates all possible errors that can occur, from network issues
/// and serialization problems to specific API-level failures. Each variant
/// contains detailed information to help with debugging and error handling.
#[derive(Error, Debug, Clone)]
pub enum RainyError {
    /// An error related to authentication.
    ///
    /// This can occur if the API key is invalid, expired, or missing.
    #[error("Authentication failed: {message}")]
    Authentication {
        /// A machine-readable error code (e.g., `INVALID_API_KEY`).
        code: String,
        /// A human-readable description of the error.
        message: String,
        /// Indicates if the request that caused this error can be retried.
        /// For authentication errors, this is typically `false`.
        retryable: bool,
    },

    /// An error indicating that the request was invalid.
    ///
    /// This usually means a required parameter was missing or a value was
    /// malformed. The `details` field may contain more specific information.
    #[error("Invalid request: {message}")]
    InvalidRequest {
        /// A machine-readable error code (e.g., `MISSING_REQUIRED_FIELD`).
        code: String,
        /// A human-readable description of the error.
        message: String,
        /// Additional details about the error, often a JSON object.
        details: Option<serde_json::Value>,
    },

    /// An error that originated from an underlying AI provider (e.g., OpenAI, Anthropic).
    ///
    /// This variant is used when the Rainy API successfully proxies a request, but the
    /// downstream provider returns an error.
    #[error("Provider error ({provider}): {message}")]
    Provider {
        /// The error code from the provider.
        code: String,
        /// The error message from the provider.
        message: String,
        /// The name of the provider that failed (e.g., `"openai"`).
        provider: String,
        /// Indicates if the request might succeed on a retry.
        retryable: bool,
    },

    /// An error indicating that the rate limit for the API has been exceeded.
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        /// A machine-readable error code, usually `RATE_LIMIT_EXCEEDED`.
        code: String,
        /// A human-readable description of the error.
        message: String,
        /// The number of seconds to wait before retrying the request, if available.
        retry_after: Option<u64>,
        /// Information about the current usage, if available.
        current_usage: Option<String>,
    },

    /// An error indicating that the account has insufficient credits to perform the request.
    #[error("Insufficient credits: {message}")]
    InsufficientCredits {
        /// A machine-readable error code, usually `INSUFFICIENT_CREDITS`.
        code: String,
        /// A human-readable description of the error.
        message: String,
        /// The current credit balance of the account.
        current_credits: f64,
        /// The estimated cost of the failed request.
        estimated_cost: f64,
        /// The date when credits are expected to be reset or replenished, if applicable.
        reset_date: Option<String>,
    },

    /// An error related to network connectivity.
    ///
    /// This can include DNS failures, connection timeouts, or other issues reaching the API server.
    #[error("Network error: {message}")]
    Network {
        /// A description of the network problem.
        message: String,
        /// Indicates if the request is likely to succeed on a retry.
        retryable: bool,
        /// The underlying error message, if available.
        source_error: Option<String>,
    },

    /// A generic error reported by the Rainy API.
    ///
    /// This is used for API errors that do not fit into the more specific categories.
    #[error("API error [{status_code}]: {message}")]
    Api {
        /// The API-specific error code.
        code: String,
        /// The error message from the API.
        message: String,
        /// The HTTP status code of the response (e.g., 500).
        status_code: u16,
        /// Indicates if the request might succeed on a retry.
        retryable: bool,
        /// The unique ID of the request, useful for support inquiries.
        request_id: Option<String>,
    },

    /// An error indicating that the request timed out.
    #[error("Request timeout: {message}")]
    Timeout {
        /// A message describing the timeout.
        message: String,
        /// The configured timeout duration in milliseconds.
        duration_ms: u64,
    },

    /// An error that occurred during JSON serialization or deserialization.
    ///
    /// This typically indicates a mismatch between the expected and actual data format.
    #[error("Serialization error: {message}")]
    Serialization {
        /// A message describing the serialization issue.
        message: String,
        /// The underlying `serde_json` error message, if available.
        source_error: Option<String>,
    },
}

impl RainyError {
    /// Returns `true` if the error is considered retryable.
    ///
    /// An error is retryable if it's a network error, a server-side (5xx) API error,
    /// a rate limit error, or a timeout. Authentication and invalid request errors
    /// are generally not retryable.
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

    /// Returns the recommended delay in seconds before retrying, if applicable.
    ///
    /// This is typically only present for `RateLimit` errors.
    pub fn retry_after(&self) -> Option<u64> {
        match self {
            RainyError::RateLimit { retry_after, .. } => *retry_after,
            _ => None,
        }
    }

    /// Returns the machine-readable error code, if available.
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

    /// Returns the unique request ID associated with the failed API call, if available.
    ///
    /// This ID is useful for debugging and for reporting issues to support.
    pub fn request_id(&self) -> Option<&str> {
        match self {
            RainyError::Api { request_id, .. } => request_id.as_deref(),
            _ => None,
        }
    }
}

/// A standard wrapper for API error responses.
///
/// The Rainy API consistently returns error objects with this structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    /// The detailed error object.
    pub error: ApiErrorDetails,
}

/// Detailed information about an API error.
///
/// This struct contains all the fields that the Rainy API can return in an error response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorDetails {
    /// The machine-readable error code (e.g., `INVALID_API_KEY`).
    pub code: String,
    /// A human-readable message describing the error.
    pub message: String,
    /// An optional JSON value containing more specific details about the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// An optional boolean indicating if the request can be retried.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    /// An optional timestamp for when the error occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// The unique ID of the request, useful for support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// A convenience `Result` type alias used throughout the SDK.
pub type Result<T> = std::result::Result<T, RainyError>;

/// Converts a `reqwest::Error` into a `RainyError`.
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

/// Converts a `serde_json::Error` into a `RainyError`.
impl From<serde_json::Error> for RainyError {
    fn from(err: serde_json::Error) -> Self {
        RainyError::Serialization {
            message: err.to_string(),
            source_error: Some(err.to_string()),
        }
    }
}

/// Converts a `reqwest::header::InvalidHeaderValue` into a `RainyError`.
impl From<reqwest::header::InvalidHeaderValue> for RainyError {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        RainyError::InvalidRequest {
            code: "INVALID_HEADER".to_string(),
            message: format!("Invalid header value: {}", err),
            details: None,
        }
    }
}
