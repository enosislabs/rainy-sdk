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

pub mod auth;
pub mod client;
pub mod error;
pub mod models;
pub mod retry;

mod endpoints;

pub use auth::AuthConfig;
pub use client::RainyClient;
pub use error::{RainyError, Result, ApiErrorResponse, ApiErrorDetails};
pub use models::*;
pub use retry::{RetryConfig, retry_with_backoff};

// Re-export commonly used types
pub use reqwest;
pub use serde_json;

/// Version of the SDK
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default base URL for the Rainy API
pub const DEFAULT_BASE_URL: &str = "https://api.enosislabs.com";
