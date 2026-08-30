//! Provider-neutral reasoning controls shared by compatible protocols.
//!
//! These types model the public `reasoning` and `reasoning_effort` controls.
//! They never infer a model family and never turn an effort label into a token
//! budget. Numeric controls are sent only when the caller explicitly supplies
//! them.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::{collections::BTreeMap, fmt};

/// Stable reasoning-effort values accepted by the compatible API surface.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReasoningEffort {
    /// Disable reasoning when the selected model supports `none`.
    None,
    /// Request minimal reasoning effort.
    Minimal,
    /// Request low reasoning effort.
    Low,
    /// Request medium reasoning effort.
    Medium,
    /// Request high reasoning effort.
    High,
    /// Request extra-high reasoning effort.
    XHigh,
    /// Request the model's maximum reasoning effort.
    Max,
    /// Preserve a future or service-defined value without remapping it.
    Custom(String),
}

impl ReasoningEffort {
    /// Returns the exact wire value.
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::XHigh => "xhigh",
            Self::Max => "max",
            Self::Custom(value) => value,
        }
    }

    /// Returns whether the value is one of the documented canonical values.
    pub fn is_canonical(&self) -> bool {
        !matches!(self, Self::Custom(_))
    }

    /// Parses a wire value while preserving unknown values.
    pub fn parse(value: impl AsRef<str>) -> Self {
        match value.as_ref() {
            "none" => Self::None,
            "minimal" => Self::Minimal,
            "low" => Self::Low,
            "medium" => Self::Medium,
            "high" => Self::High,
            "xhigh" => Self::XHigh,
            "max" => Self::Max,
            other => Self::Custom(other.to_string()),
        }
    }
}

impl fmt::Display for ReasoningEffort {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl From<&str> for ReasoningEffort {
    fn from(value: &str) -> Self {
        Self::parse(value)
    }
}

impl From<String> for ReasoningEffort {
    fn from(value: String) -> Self {
        Self::parse(value)
    }
}

impl Serialize for ReasoningEffort {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for ReasoningEffort {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Self::parse(String::deserialize(deserializer)?))
    }
}

/// Explicit fields accepted in a structured `reasoning` object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ReasoningConfig {
    /// Explicitly enable or disable reasoning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Effort label supplied exactly as selected by the caller.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,
    /// Explicit numeric reasoning-token budget, when the protocol supports it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Exclude reasoning from the returned content, when supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<bool>,
    /// Explicit extension fields inside the reasoning object.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl ReasoningConfig {
    /// Creates an adaptive/default reasoning request.
    pub fn adaptive() -> Self {
        Self {
            enabled: Some(true),
            ..Self::default()
        }
    }

    /// Creates an effort request without deriving a numeric budget.
    pub fn effort(effort: impl Into<ReasoningEffort>) -> Self {
        Self {
            effort: Some(effort.into()),
            ..Self::default()
        }
    }

    /// Creates an explicit numeric budget request.
    pub fn manual_budget(max_tokens: u32) -> Self {
        Self {
            max_tokens: Some(max_tokens),
            ..Self::default()
        }
    }

    /// Creates a request that asks the service to omit reasoning where supported.
    pub fn excluded() -> Self {
        Self {
            exclude: Some(true),
            ..Self::default()
        }
    }

    /// Sets the explicit enabled flag.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Sets an exact effort label.
    pub fn with_effort(mut self, effort: impl Into<ReasoningEffort>) -> Self {
        self.effort = Some(effort.into());
        self
    }

    /// Sets an exact numeric token budget.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the exclusion flag.
    pub fn with_exclude(mut self, exclude: bool) -> Self {
        self.exclude = Some(exclude);
        self
    }

    /// Adds an explicit extension field.
    pub fn with_extra(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }

    /// Validates unambiguous reasoning controls.
    pub fn validate(&self) -> Result<(), String> {
        if self.effort.is_some() && self.max_tokens.is_some() {
            return Err(
                "reasoning.effort and reasoning.max_tokens cannot be sent together".to_string(),
            );
        }
        if self.max_tokens == Some(0) {
            return Err("reasoning.max_tokens must be greater than zero".to_string());
        }
        Ok(())
    }
}

/// The boolean, structured, or raw reasoning form accepted by compatible APIs.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ReasoningRequest {
    /// Boolean reasoning toggle.
    Enabled(bool),
    /// Structured reasoning object.
    Config(ReasoningConfig),
    /// Deliberately preserved open compatibility value.
    Raw(Value),
}

impl From<bool> for ReasoningRequest {
    fn from(value: bool) -> Self {
        Self::Enabled(value)
    }
}

impl From<ReasoningConfig> for ReasoningRequest {
    fn from(value: ReasoningConfig) -> Self {
        Self::Config(value)
    }
}

impl From<ReasoningEffort> for ReasoningRequest {
    fn from(value: ReasoningEffort) -> Self {
        Self::Config(ReasoningConfig::effort(value))
    }
}

impl From<Value> for ReasoningRequest {
    fn from(value: Value) -> Self {
        match value {
            Value::Bool(value) => Self::Enabled(value),
            Value::Object(object) => {
                let original = Value::Object(object.clone());
                serde_json::from_value(original)
                    .map(Self::Config)
                    .unwrap_or(Self::Raw(Value::Object(object)))
            }
            other => Self::Raw(other),
        }
    }
}

/// High-level reasoning intent for callers that want a typed choice.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReasoningControl {
    /// Let the selected model use adaptive/default reasoning.
    Adaptive,
    /// Use an explicitly selected effort value.
    Effort(ReasoningEffort),
    /// Use an explicitly selected numeric budget.
    ManualBudget(u32),
    /// Ask the service to disable or exclude reasoning.
    Disabled,
}

impl ReasoningControl {
    /// Converts the intent to a literal structured reasoning object.
    pub fn as_config(&self) -> ReasoningConfig {
        match self {
            Self::Adaptive => ReasoningConfig::adaptive(),
            Self::Effort(effort) => ReasoningConfig::effort(effort.clone()),
            Self::ManualBudget(tokens) => ReasoningConfig::manual_budget(*tokens),
            Self::Disabled => ReasoningConfig::excluded(),
        }
    }
}

/// A numeric budget choice declared by a model or selected explicitly.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasoningBudget {
    /// A concrete token count.
    Tokens(u32),
    /// A model-declared dynamic value that this SDK does not invent.
    Dynamic,
    /// A model-declared disabled value.
    Disabled,
}

impl ReasoningBudget {
    /// Creates a concrete budget.
    pub const fn tokens(value: u32) -> Self {
        Self::Tokens(value)
    }

    /// Returns a concrete token count, if one exists.
    pub const fn as_tokens(self) -> Option<u32> {
        match self {
            Self::Tokens(value) => Some(value),
            Self::Dynamic | Self::Disabled => None,
        }
    }
}
