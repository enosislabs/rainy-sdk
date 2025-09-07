use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The comprehensive error type for all operations within the Rainy SDK.
///
/// `RainyError` is an enumeration of all possible errors that can occur,
/// providing detailed context for each error variant.
#[derive(Error, Debug, Clone)]
pub enum RainyError {
    /// An error related to authentication, such as an invalid or expired API key.
    #[error("Authentication failed: {message}")]
    Authentication {
        /// A machine-readable error code (e.g., `INVALID_API_KEY`).
        code: String,
        /// A human-readable error message.
        message: String,
        /// Indicates whether the request can be retried.
        retryable: bool,
    },

    /// An error due to an invalid request, such as a missing required field.
    #[error("Invalid request: {message}")]
    InvalidRequest {
        /// A machine-readable error code (e.g., `MISSING_REQUIRED_FIELD`).
        code: String,
        /// A human-readable error message.
        message: String,
        /// Additional details about the error, if available.
        details: Option<serde_json::Value>,
    },

    /// An error that originates from an underlying AI provider (e.g., OpenAI, Anthropic).
    #[error("Provider error ({provider}): {message}")]
    Provider {
        /// The error code from the provider.
        code: String,
        /// The error message from the provider.
        message: String,
        /// The name of the provider that returned the error.
        provider: String,
        /// Indicates whether the request can be retried.
        retryable: bool,
    },

    /// An error indicating that the rate limit for the API has been exceeded.
    #[error("Rate limit exceeded: {message}")]
    RateLimit {
        /// A machine-readable error code (e.g., `RATE_LIMIT_EXCEEDED`).
        code: String,
        /// A human-readable error message.
        message: String,
        /// The recommended time to wait before retrying, in seconds.
        retry_after: Option<u64>,
        /// Information about the current usage, if available.
        current_usage: Option<String>,
    },

    /// An error indicating that the account has insufficient credits to perform the request.
    #[error("Insufficient credits: {message}")]
    InsufficientCredits {
        /// A machine-readable error code (e.g., `INSUFFICIENT_CREDITS`).
        code: String,
        /// A human-readable error message.
        message: String,
        /// The current credit balance of the account.
        current_credits: f64,
        /// The estimated cost of the request.
        estimated_cost: f64,
        /// The date when the credits are scheduled to be reset or topped up.
        reset_date: Option<String>,
    },

    /// An error related to network connectivity or HTTP-level issues.
    #[error("Network error: {message}")]
    Network {
        /// A message describing the network error.
        message: String,
        /// Indicates whether the request can be retried.
        retryable: bool,
        /// The underlying error message, if available.
        source_error: Option<String>,
    },

    /// A generic API error that doesn't fit into the other categories.
    #[error("API error [{status_code}]: {message}")]
    Api {
        /// A machine-readable error code.
        code: String,
        /// A human-readable error message.
        message: String,
        /// The HTTP status code of the response.
        status_code: u16,
        /// Indicates whether the request can be retried.
        retryable: bool,
        /// The unique ID of the request, for debugging purposes.
        request_id: Option<String>,
    },

    /// An error indicating that the request timed out.
    #[error("Request timeout: {message}")]
    Timeout {
        /// A message describing the timeout.
        message: String,
        /// The timeout duration in milliseconds.
        duration_ms: u64,
    },

    /// An error that occurs during serialization or deserialization of data.
    #[error("Serialization error: {message}")]
    Serialization {
        /// A message describing the serialization error.
        message: String,
        /// The underlying error message, if available.
        source_error: Option<String>,
    },
}

impl RainyError {
    /// Checks if the error is considered retryable.
    ///
    /// Some errors, like network issues or rate limiting, are transient and can be resolved
    /// by retrying the request.
    ///
    /// # Returns
    ///
    /// `true` if the error is retryable, `false` otherwise.
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

    /// Returns the recommended delay in seconds before a retry, if applicable.
    ///
    /// This is typically used with `RateLimit` errors.
    ///
    /// # Returns
    ///
    /// An `Option<u64>` containing the retry delay in seconds, or `None` if not applicable.
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

    /// Returns the unique request ID associated with the error, if available.
    ///
    /// This is useful for debugging and support requests.
    pub fn request_id(&self) -> Option<&str> {
        match self {
            RainyError::Api { request_id, .. } => request_id.as_deref(),
            _ => None,
        }
    }
}

/// The structure of a standard error response from the Rainy API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    /// The detailed error information.
    pub error: ApiErrorDetails,
}

/// Detailed information about an API error.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorDetails {
    /// A machine-readable error code.
    pub code: String,
    /// A human-readable error message.
    pub message: String,
    /// Additional, structured details about the error.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    /// Indicates whether the request that caused this error can be retried.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retryable: Option<bool>,
    /// The timestamp of when the error occurred.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
    /// The unique ID of the request.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

/// A convenience type alias for `Result<T, RainyError>`.
pub type Result<T> = std::result::Result<T, RainyError>;

/// Converts a `reqwest::Error` into a `RainyError`.
///
/// This implementation categorizes `reqwest` errors into `Timeout`, `Network`,
/// or other appropriate `RainyError` variants.
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
///
/// This is used for errors that occur during the serialization or deserialization of JSON data.
impl From<serde_json::Error> for RainyError {
    fn from(err: serde_json::Error) -> Self {
        RainyError::Serialization {
            message: err.to_string(),
            source_error: Some(err.to_string()),
        }
    }
}

/// Converts a `reqwest::header::InvalidHeaderValue` into a `RainyError`.
///
/// This is used when an invalid value is provided for an HTTP header.
impl From<reqwest::header::InvalidHeaderValue> for RainyError {
    fn from(err: reqwest::header::InvalidHeaderValue) -> Self {
        RainyError::InvalidRequest {
            code: "INVALID_HEADER".to_string(),
            message: format!("Invalid header value: {}", err),
            details: None,
        }
    }
}
