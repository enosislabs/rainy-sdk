//! Typed shared Chat request controls.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Chat `stop` value accepted by Rainy: one string or up to four strings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum StopSequences {
    /// One stop sequence.
    One(String),
    /// Multiple stop sequences.
    Many(Vec<String>),
}

impl From<String> for StopSequences {
    fn from(value: String) -> Self {
        Self::One(value)
    }
}

impl From<&str> for StopSequences {
    fn from(value: &str) -> Self {
        Self::One(value.to_string())
    }
}

impl From<Vec<String>> for StopSequences {
    fn from(value: Vec<String>) -> Self {
        Self::Many(value)
    }
}

impl StopSequences {
    /// Returns all configured sequences as a borrowed slice-like iterator.
    pub fn as_slice(&self) -> Vec<&str> {
        match self {
            Self::One(value) => vec![value],
            Self::Many(values) => values.iter().map(String::as_str).collect(),
        }
    }
}

/// Provider routing options forwarded as Rainy's `provider` object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProviderOptions {
    /// Preferred provider order.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,
    /// Whether the router may fall back to another provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,
    /// Require the provider to support all request parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_parameters: Option<bool>,
    /// Provider sorting mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    /// Data-collection preference when the upstream router supports it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_collection: Option<String>,
    /// Provider quantization preferences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantizations: Option<Vec<String>>,
    /// Explicit future provider fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl ProviderOptions {
    /// Creates routing options that prefer one provider.
    pub fn preferred(provider: impl Into<String>) -> Self {
        Self {
            order: Some(vec![provider.into()]),
            ..Self::default()
        }
    }

    /// Sets an explicit provider preference order.
    pub fn with_order<I, S>(mut self, providers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.order = Some(providers.into_iter().map(Into::into).collect());
        self
    }

    /// Sets whether upstream provider fallback is allowed.
    pub fn with_allow_fallbacks(mut self, allow: bool) -> Self {
        self.allow_fallbacks = Some(allow);
        self
    }

    /// Adds an explicit provider extension field.
    pub fn with_extra(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }
}

impl From<String> for ProviderOptions {
    fn from(value: String) -> Self {
        Self::preferred(value)
    }
}

impl From<&str> for ProviderOptions {
    fn from(value: &str) -> Self {
        Self::preferred(value)
    }
}

/// Stream controls accepted by the Chat route.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChatStreamOptions {
    /// Include a final usage chunk when the provider supports it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>,
    /// Explicit future stream fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl ChatStreamOptions {
    /// Creates stream options requesting usage in the terminal chunk.
    pub fn include_usage() -> Self {
        Self {
            include_usage: Some(true),
            ..Self::default()
        }
    }

    /// Adds an explicit future stream option.
    pub fn with_extra(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }
}

/// Output modality accepted by Chat.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ChatModality {
    /// Text output.
    Text,
    /// Audio output.
    Audio,
    /// Image output.
    Image,
}

/// Typed image-generation options accepted by Chat when image output is requested.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChatImageConfig {
    /// Requested output size, if supported by the provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Requested image quality, if supported by the provider.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// Explicit provider-specific image controls.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl ChatImageConfig {
    /// Sets the requested image size.
    pub fn with_size(mut self, size: impl Into<String>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Sets the requested image quality.
    pub fn with_quality(mut self, quality: impl Into<String>) -> Self {
        self.quality = Some(quality.into());
        self
    }
}
