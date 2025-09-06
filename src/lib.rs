//! # 🌧️ Rainy SDK
//!
//! [![Crates.io](https://img.shields.io/crates/v/rainy-sdk.svg)](https://crates.io/crates/rainy-sdk)
//! [![Documentation](https://docs.rs/rainy-sdk/badge.svg)](https://docs.rs/rainy-sdk)
//! [![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
//!
//! The official Rust SDK for the **Rainy API by Enosis Labs** - a unified interface for multiple AI providers including OpenAI, Anthropic, Google Gemini, and more.
//!
//! This SDK provides a clean, idiomatic Rust interface for interacting with
//! the Rainy API, which unifies multiple AI providers under a single API, offering
//! features like intelligent retries, metadata tracking, and comprehensive error handling.
//!
//! ## ✨ Features
//!
//! - **🚀 Unified API**: A single, consistent interface for multiple AI providers.
//! - **🔐 Type-Safe Authentication**: Securely manage API keys with compile-time checks.
//! - **⚡ Async/Await**: Built with full support for modern asynchronous Rust using Tokio.
//! - **📊 Rich Metadata**: Get detailed response metadata, including response times, provider info, token usage, and credit tracking.
//! - **🛡️ Enhanced Error Handling**: A comprehensive set of error types to handle API failures gracefully, with built-in retry logic.
//! - **🔄 Intelligent Retry**: Automatic exponential backoff with jitter for resilient communication.
//! - **📈 Rate Limiting**: Optional client-side rate limiting to prevent hitting API limits.
//! - **📚 Rich Documentation**: Complete API documentation with practical, runnable examples.
//!
//! ## 📦 Installation
//!
//! Add this to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! rainy-sdk = "0.2.0"
//! tokio = { version = "1.0", features = ["full"] }
//! ```
//!
//! Or install with cargo:
//!
//! ```bash
//! cargo add rainy-sdk
//! ```
//!
//! ### Optional Features
//!
//! Enable additional features as needed:
//!
//! ```toml
//! [dependencies]
//! rainy-sdk = { version = "0.2.0", features = ["rate-limiting", "tracing"] }
//! ```
//!
//! - `rate-limiting`: Enables client-side rate limiting using the `governor` crate.
//! - `tracing`: Enables logging of requests and responses using the `tracing` crate.
//!
//! ## 🚀 Quick Start
//!
//! ```rust,no_run
//! use rainy_sdk::{RainyClient, models::{self, ChatMessage}};
//! use std::error::Error;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     // Initialize the client with your API key from environment variables or directly.
//!     let client = RainyClient::with_api_key("ra-your-api-key")?;
//!
//!     // Perform a simple health check to verify connectivity.
//!     let health = client.health_check().await?;
//!     println!("API Status: {:?}", health.status);
//!
//!     // Send a simple chat message.
//!     let response = client.simple_chat(models::model_constants::GPT_4O, "Hello! Tell me a short story.").await?;
//!     println!("AI Response: {}", response);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Authentication
//!
//! The SDK authenticates using an API key. You can create a `RainyClient` instance
//! by providing the key directly or by setting the `RAINY_API_KEY` environment variable.
//!
//! ```rust,no_run
//! # use rainy_sdk::RainyClient;
//! # use std::env;
//! #
//! # fn main() -> Result<(), rainy_sdk::RainyError> {
//! // With API key directly
//! let client_from_key = RainyClient::with_api_key("ra-your-api-key")?;
//!
//! // From environment variable (recommended)
//! env::set_var("RAINY_API_KEY", "ra-your-api-key");
//! let client_from_env = RainyClient::new()?;
//! # Ok(())
//! # }
//! ```

/// The `auth` module provides structures and methods for handling API authentication.
///
/// It includes `AuthConfig` for configuring authentication details like API keys
/// and base URLs. This module is central to how the `RainyClient` authenticates
/// with the Rainy API.
pub mod auth;

/// The `client` module contains the main `RainyClient`.
///
/// `RainyClient` is the primary entry point for interacting with the Rainy API.
/// It encapsulates all the logic for making requests, handling authentication,
/// and managing responses.
pub mod client;

/// The `error` module defines the error types used throughout the SDK.
///
/// It includes the main `RainyError` enum, which covers everything from
/// network issues to API-specific errors. It also provides detailed
/// error structures for better diagnostics.
pub mod error;

/// The `models` module contains all the data structures for API requests and responses.
///
/// This includes request builders for chat completions, user information, API keys,
/// and various response formats. It also defines constants for supported AI models.
pub mod models;

/// The `retry` module provides utilities for implementing retry logic.
///
/// It includes `RetryConfig` for configuring retry behavior (like number of attempts
/// and backoff strategy) and a `retry_with_backoff` function to wrap API calls
/// with retry capabilities.
pub mod retry;

/// Internal module for API endpoint implementations.
mod endpoints;

// Re-export core components for easier access.
pub use auth::AuthConfig;
pub use client::RainyClient;
pub use error::{ApiErrorDetails, ApiErrorResponse, RainyError, Result};
pub use models::*;
pub use retry::{retry_with_backoff, RetryConfig};

// Re-export commonly used third-party crates for user convenience.
/// Re-export of the `reqwest` crate.
///
/// This is provided for convenience, allowing users to access `reqwest` types
/// that might be needed for advanced customization or for handling raw responses,
/// without adding `reqwest` to their own `Cargo.toml`.
pub use reqwest;

/// Re-export of the `serde_json` crate.
///
/// This is provided for convenience, enabling users to work with JSON values
/// directly, for example, when dealing with raw JSON responses or creating
/// custom request payloads.
pub use serde_json;

/// The current version of the Rainy SDK, read from `Cargo.toml`.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The default base URL for the Rainy API.
///
/// This constant is used by the `RainyClient` when no other base URL is specified.
/// It points to the main production endpoint of the Rainy API.
pub const DEFAULT_BASE_URL: &str = "https://api.enosislabs.com";
