//! OpenAI-compatible embeddings models.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Input accepted by an embeddings endpoint.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum EmbeddingInput {
    /// One text string.
    Text(String),
    /// Multiple text strings.
    Texts(Vec<String>),
    /// One tokenized input.
    Tokens(Vec<u32>),
    /// Multiple tokenized inputs.
    TokenBatches(Vec<Vec<u32>>),
}

/// Encoding requested for returned embeddings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingEncodingFormat {
    /// JSON floating-point arrays.
    Float,
    /// Base64-encoded vectors.
    Base64,
}

/// OpenAI-compatible embeddings request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingsRequest {
    /// Model identifier.
    pub model: String,
    /// Text or token input.
    pub input: EmbeddingInput,
    /// Output encoding.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub encoding_format: Option<EmbeddingEncodingFormat>,
    /// Requested output dimensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dimensions: Option<u32>,
    /// End-user identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Optional metadata accepted by compatible services.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, String>>,
    /// Explicit future-compatible fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl EmbeddingsRequest {
    /// Creates a request for one text input.
    pub fn text(model: impl Into<String>, input: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            input: EmbeddingInput::Text(input.into()),
            encoding_format: None,
            dimensions: None,
            user: None,
            metadata: None,
            extra: BTreeMap::new(),
        }
    }

    /// Sets the output encoding.
    pub fn with_encoding_format(mut self, encoding_format: EmbeddingEncodingFormat) -> Self {
        self.encoding_format = Some(encoding_format);
        self
    }

    /// Sets an output dimension request.
    pub fn with_dimensions(mut self, dimensions: u32) -> Self {
        self.dimensions = Some(dimensions);
        self
    }

    /// Sets the end-user identifier.
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Adds a metadata key/value pair.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata
            .get_or_insert_with(BTreeMap::new)
            .insert(key.into(), value.into());
        self
    }

    /// Adds an explicit future-compatible field.
    pub fn with_extra(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }

    /// Validates protocol-level embedding request constraints.
    pub fn validate(&self) -> Result<(), String> {
        if self.dimensions == Some(0) {
            return Err("dimensions must be greater than zero".to_string());
        }
        Ok(())
    }
}

/// Returned embedding vector or base64 value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum EmbeddingValue {
    /// Floating-point vector.
    Float(Vec<f32>),
    /// Base64 vector.
    Base64(String),
}

/// One embedding result.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingData {
    /// Object label.
    pub object: String,
    /// Input position.
    pub index: u32,
    /// Returned vector.
    pub embedding: EmbeddingValue,
}

/// Embeddings token usage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct EmbeddingsUsage {
    /// Input token count.
    #[serde(default, alias = "input_tokens")]
    pub prompt_tokens: u64,
    /// Total token count.
    #[serde(default)]
    pub total_tokens: u64,
}

/// OpenAI-compatible embeddings response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingsResponse {
    /// Object label.
    pub object: String,
    /// Returned vectors.
    pub data: Vec<EmbeddingData>,
    /// Model used.
    pub model: String,
    /// Token usage.
    pub usage: EmbeddingsUsage,
    /// Future-compatible response fields.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}
