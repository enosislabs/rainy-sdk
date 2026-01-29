//! # Rainy SDK
//!
//! The official Rust SDK for the Rainy API by Enosis Labs.
//!
//! This SDK provides a clean, idiomatic Rust interface for interacting with
//! the Rainy API, which unifies multiple AI providers under a single API.
//!
//! ## Features
//!
//! - **Idiomatic Rust API**: Clean, type-safe interfaces
//! - **Automatic Authentication**: API key and admin key management
//! - **Rate Limiting**: Built-in rate limit handling
//! - **Error Handling**: Comprehensive error types and handling
//! - **Async Support**: Full async/await support with Tokio
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use rainy_sdk::RainyClient;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create client with API key - automatically connects to api.enosislabs.com
//!     let client = RainyClient::with_api_key("your-api-key")?;
//!
//!     // Check API health
//!     let health = client.health_check().await?;
//!     println!("API Status: {:?}", health.status);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Authentication
//!
//! The SDK uses API key authentication:
//!
//! ### API Key Authentication
//!
//! ```rust,no_run
//! # use rainy_sdk::RainyClient;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Simplest way to create a client
//! let client = RainyClient::with_api_key("ra-20250125143052Ab3Cd9Ef2Gh5Ik8Lm4Np7Qr")?;
//! # Ok(())
//! # }
//! ```
//!

/// Handles authentication and API key management.
pub mod auth;
/// The main client for interacting with the Rainy API.
pub mod client;
/// Cowork integration: tiers, capabilities, and feature gating.
pub mod cowork;
/// Defines error types and result aliases for the SDK.
pub mod error;
/// Contains the data models for API requests and responses.
pub mod models;
/// Implements retry logic with exponential backoff.
pub mod retry;
/// Web search types and options for Tavily-powered search.
pub mod search;

mod endpoints;

pub use auth::AuthConfig;
pub use client::RainyClient;
pub use error::{ApiErrorDetails, ApiErrorResponse, RainyError, Result};
pub use models::*;
pub use retry::{retry_with_backoff, RetryConfig};

// Re-export Cowork types for convenience
pub use cowork::{CoworkCapabilities, CoworkFeatures, CoworkPlan, CoworkUsage};
// Backward compatibility aliases
// #[allow(deprecated)]
// pub use cowork::{CoworkLimits, CoworkTier};
pub use endpoints::cowork::get_offline_capabilities;

// Re-export Research types for convenience
pub use search::{
    DeepResearchResponse, ResearchApiResponse, ResearchConfig, ResearchResponse, ResearchResult,
    ResearchSource,
};

// Re-export commonly used types
/// Re-export of the `reqwest` crate for convenience.
///
/// This allows users of the SDK to use `reqwest` types without adding it
/// as a direct dependency to their project.
pub use reqwest;
/// Re-export of the `serde_json` crate for convenience.
///
/// This allows users of the SDK to use `serde_json` types for serialization
/// and deserialization without adding it as a direct dependency.
pub use serde_json;

/// The current version of the Rainy SDK.
///
/// This value is read from the `CARGO_PKG_VERSION` environment variable at compile time.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The default base URL for the Rainy API.
///
/// This constant is used by the `RainyClient` as the default API endpoint.
pub const DEFAULT_BASE_URL: &str = "https://rainy-api-v2-179843975974.us-west1.run.app";
