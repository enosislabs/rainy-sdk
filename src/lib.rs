//! # Rainy SDK
//!
//! A universal Rust SDK for OpenAI-compatible and Anthropic-compatible inference APIs.
//!
//! This SDK provides a clean, idiomatic Rust interface for interacting with
//! Rainy and other services that expose compatible inference protocols.
//!
//! ## Features
//!
//! - **Idiomatic Rust API**: Clean, type-safe interfaces
//! - **Protocol boundaries**: Chat, Responses, Messages, and embeddings
//! - **Optional extensions**: Explicit Rainy model discovery, health, and search helpers
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
//!     // Create a client with an API key. No discovery call is made here.
//!     let client = RainyClient::with_api_key("your-api-key")?;
//!
//!     let response = client
//!         .simple_chat("compatible/chat-model", "Say hello")
//!         .await?;
//!     println!("{response}");
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Authentication
//!
//! API-key authentication is selected by protocol. OpenAI-compatible routes
//! use Bearer authentication; native Anthropic Messages routes use `x-api-key`
//! and `anthropic-version` headers.
//!
//! ### API Key Authentication
//!
//! ```rust,no_run
//! # use rainy_sdk::RainyClient;
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Simplest way to create a client
//! let client = RainyClient::with_api_key("your-api-key")?;
//! # Ok(())
//! # }
//! ```
//!

/// Handles protocol-neutral API-key authentication and request safety checks.
pub mod auth;
/// The main client for protocol-compatible inference and explicit Rainy extensions.
pub mod client;
/// Defines error types and result aliases for the SDK.
pub mod error;
/// Contains the data models for API requests and responses.
pub mod models;
/// Implements retry logic with exponential backoff.
pub mod retry;
/// Web search and research types for the optional Rainy search extension.
pub mod search;
/// Optional JWT/session client for Rainy account endpoints.
#[cfg(feature = "rainy-account")]
pub mod session;

mod sse;

mod endpoints;

/// Default native Anthropic API version used by Messages requests.
pub use endpoints::messages::DEFAULT_ANTHROPIC_VERSION;

pub use auth::AuthConfig;
pub use client::RainyClient;
pub use error::{ApiErrorDetails, ApiErrorResponse, RainyError, Result};
pub use models::*;
pub use retry::{RetryConfig, retry_with_backoff};
#[cfg(feature = "rainy-account")]
pub use session::{
    CreatedApiKey, LoginResponse, OrgProfile, RainySessionClient, RefreshResponse,
    SessionApiKeyListItem, SessionConfig, SessionTokens, SessionUser, UsageCreditsResponse,
    UsageStatsResponse,
};

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

/// The default base URL for the Rainy API v3 service.
///
/// Note: the v3 service currently exposes its canonical HTTP API under `/api/v1/*`.
pub const DEFAULT_BASE_URL: &str = "https://rainy-api-v3-us-160298401329.us-east4.run.app";
