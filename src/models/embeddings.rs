//! Embeddings domain facade.

/// Embeddings types remain re-exported from the compatibility root while this
/// module provides a stable domain import path.
pub use super::{
    EmbeddingData, EmbeddingEncodingFormat, EmbeddingInput, EmbeddingValue, EmbeddingsRequest,
    EmbeddingsResponse, EmbeddingsUsage,
};
