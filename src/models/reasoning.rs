//! Typed reasoning controls shared by Chat, Responses, and model discovery.
//!
//! Rainy deliberately forwards the provider-neutral OpenRouter reasoning
//! contract. The SDK therefore models only controls that are part of that
//! contract (`enabled`, `effort`, `max_tokens`, and `exclude`). Provider or
//! model-specific controls must be carried in the explicit extension map and
//! must be selected from the model catalog rather than inferred from a model
//! name.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::collections::BTreeMap;
use std::fmt;

/// Canonical reasoning-effort values accepted by Rainy.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ReasoningEffort {
    /// Disable reasoning when the model supports an explicit `none` value.
    None,
    /// Request the smallest supported reasoning effort.
    Minimal,
    /// Request low reasoning effort.
    Low,
    /// Request medium reasoning effort.
    Medium,
    /// Request high reasoning effort.
    High,
    /// Request extra-high reasoning effort.
    XHigh,
    /// Request the maximum reasoning effort.
    Max,
    /// Preserve a future catalog/provider value without silently remapping it.
    Custom(String),
}

impl ReasoningEffort {
    /// Returns the exact wire value for this effort.
    pub fn as_str(&self) -> &str {
        match self {
            Self::None => "none",
            Self::Minimal => "minimal",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::XHigh => "xhigh",
            Self::Max => "max",
            Self::Custom(value) => value.as_str(),
        }
    }

    /// Returns whether the value is one of Rainy's canonical effort levels.
    pub fn is_canonical(&self) -> bool {
        !matches!(self, Self::Custom(_))
    }

    /// Parses an effort value while preserving unknown values.
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
        let value = String::deserialize(deserializer)?;
        Ok(Self::parse(value))
    }
}

/// A typed reasoning configuration for Rainy Chat and Responses requests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ReasoningConfig {
    /// Explicitly enable or disable reasoning when the model exposes a toggle.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    /// Canonical or catalog-declared effort value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,
    /// Explicit reasoning-token budget when the selected model declares this control.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Exclude reasoning from provider output/processing when supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<bool>,
    /// Explicit provider/model extension fields inside the reasoning object.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl ReasoningConfig {
    /// Creates an empty configuration that enables the model's adaptive/default reasoning.
    pub fn adaptive() -> Self {
        Self {
            enabled: Some(true),
            ..Self::default()
        }
    }

    /// Creates an effort-based reasoning configuration.
    pub fn effort(effort: impl Into<ReasoningEffort>) -> Self {
        Self {
            effort: Some(effort.into()),
            ..Self::default()
        }
    }

    /// Creates a manual numeric reasoning-budget configuration.
    pub fn manual_budget(max_tokens: u32) -> Self {
        Self {
            max_tokens: Some(max_tokens),
            ..Self::default()
        }
    }

    /// Creates an exclusion configuration.
    pub fn excluded() -> Self {
        Self {
            exclude: Some(true),
            ..Self::default()
        }
    }

    /// Sets the enabled flag.
    pub fn with_enabled(mut self, enabled: bool) -> Self {
        self.enabled = Some(enabled);
        self
    }

    /// Sets the effort control.
    pub fn with_effort(mut self, effort: impl Into<ReasoningEffort>) -> Self {
        self.effort = Some(effort.into());
        self
    }

    /// Sets the numeric reasoning-token budget.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the exclusion flag.
    pub fn with_exclude(mut self, exclude: bool) -> Self {
        self.exclude = Some(exclude);
        self
    }

    /// Adds an explicit extension field to the reasoning object.
    pub fn with_extra(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }

    /// Checks for controls that are mutually exclusive under Rainy's contract.
    pub fn validate(&self) -> Result<(), String> {
        if self.effort.is_some() && self.max_tokens.is_some() {
            return Err(
                "reasoning.effort and reasoning.max_tokens cannot be sent together".to_string(),
            );
        }
        Ok(())
    }
}

/// Chat/Responses reasoning payload, including the legacy boolean form.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ReasoningRequest {
    /// Boolean reasoning toggle accepted by Chat compatibility routes.
    Enabled(bool),
    /// Structured Rainy/OpenRouter reasoning controls.
    Config(ReasoningConfig),
    /// A deliberately preserved compatibility value for callers migrating from v0.x.
    /// New code should use [`ReasoningConfig`] instead.
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

/// High-level reasoning intent used when building a request from capabilities.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReasoningControl {
    /// Let the selected model choose its adaptive/default reasoning behavior.
    Adaptive,
    /// Use an explicit effort value declared by the selected model.
    Effort(ReasoningEffort),
    /// Use an explicit numeric budget declared by the selected model.
    ManualBudget(u32),
    /// Ask the selected model to exclude reasoning where supported.
    Disabled,
}

impl ReasoningControl {
    /// Converts this intent into the provider-neutral Rainy reasoning object.
    pub fn as_config(&self) -> ReasoningConfig {
        match self {
            Self::Adaptive => ReasoningConfig::adaptive(),
            Self::Effort(effort) => ReasoningConfig::effort(effort.clone()),
            Self::ManualBudget(max_tokens) => ReasoningConfig::manual_budget(*max_tokens),
            Self::Disabled => ReasoningConfig::excluded(),
        }
    }
}

/// A model-declared numeric reasoning-budget choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReasoningBudget {
    /// A concrete number of reasoning tokens.
    Tokens(u32),
    /// The model's catalog-declared dynamic budget value.
    Dynamic,
    /// The model's catalog-declared disabled value.
    Disabled,
}

impl ReasoningBudget {
    /// Creates a concrete token budget.
    pub const fn tokens(value: u32) -> Self {
        Self::Tokens(value)
    }

    /// Returns the concrete value, if this choice is already numeric.
    pub const fn as_tokens(self) -> Option<u32> {
        match self {
            Self::Tokens(value) => Some(value),
            Self::Dynamic | Self::Disabled => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_canonical_effort_and_manual_budget_without_remapping() {
        let effort = serde_json::to_value(ReasoningConfig::effort(ReasoningEffort::XHigh))
            .expect("effort JSON");
        assert_eq!(effort["effort"], "xhigh");

        let budget =
            serde_json::to_value(ReasoningConfig::manual_budget(2048)).expect("budget JSON");
        assert_eq!(budget["max_tokens"], 2048);

        let custom = serde_json::to_value(ReasoningEffort::Custom("provider_next".to_string()))
            .expect("custom JSON");
        assert_eq!(custom, "provider_next");
    }

    #[test]
    fn rejects_conflicting_effort_and_budget() {
        let config = ReasoningConfig::effort(ReasoningEffort::High).with_max_tokens(1024);
        assert!(config.validate().is_err());
    }
}
