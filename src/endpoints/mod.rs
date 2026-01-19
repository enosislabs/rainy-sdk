//! # API Endpoints
//!
//! This module contains the various API endpoints that the `RainyClient` can interact with.
//! Each submodule corresponds to a specific area of the Rainy API.

/// Endpoint for chat completions.
pub mod chat;
/// Endpoint for Cowork capabilities and tier management.
pub mod cowork;
/// Endpoint for checking the health of the API.
pub mod health;
/// Endpoint for managing API keys.
pub mod keys;
/// Endpoint for web search via Tavily.
pub mod search;
/// Endpoint for retrieving usage statistics.
pub mod usage;
/// Endpoint for managing user information.
pub mod user;
