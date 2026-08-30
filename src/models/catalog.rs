//! Optional Rainy model-discovery and capability helpers.
//!
//! These types describe public inference metadata only. They do not expose
//! account entitlements, private routes, provider topology, or organization
//! policy decisions.

use super::{ReasoningConfig, ReasoningEffort};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{cmp::Ordering, collections::BTreeMap};

/// A capability represented as a boolean or a service-defined status string.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum CapabilityFlag {
    /// Known boolean capability.
    Bool(bool),
    /// Future or unavailable capability status.
    Text(String),
}

/// Public model architecture metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelArchitecture {
    /// Input modalities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_modalities: Vec<String>,
    /// Output modalities.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub output_modalities: Vec<String>,
    /// Tokenizer label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenizer: Option<String>,
    /// Instruction format label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruct_type: Option<String>,
}

/// Small public capability summary useful for optional local selection.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RainyCapabilities {
    /// Reasoning support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<CapabilityFlag>,
    /// Image-input support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_input: Option<CapabilityFlag>,
    /// Tool-call support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<CapabilityFlag>,
    /// Structured-output support.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<CapabilityFlag>,
}

/// Public model pricing metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelPricing {
    /// Prompt price as returned by the catalog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Completion price as returned by the catalog.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<String>,
}

/// One OpenAI-compatible `/models` item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelListItem {
    /// Model identifier.
    pub id: String,
    /// Object label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// Creation timestamp.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub created: Option<u64>,
    /// Opaque owner label.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owned_by: Option<String>,
    /// Future model fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// OpenAI-compatible model-list response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelList {
    /// Object label, normally `list`.
    #[serde(default)]
    pub object: String,
    /// Model records.
    #[serde(default)]
    pub data: Vec<ModelListItem>,
    /// Future model-list fields.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

/// Public inference model-catalog item.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelCatalogItem {
    /// Model identifier.
    pub id: String,
    /// Human-readable name.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Maximum context length.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u32>,
    /// Pricing metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pricing: Option<ModelPricing>,
    /// Parameter names publicly declared by the model.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supported_parameters: Option<Vec<String>>,
    /// Architecture metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub architecture: Option<ModelArchitecture>,
    /// Small Rainy inference capability summary.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rainy_capabilities: Option<RainyCapabilities>,
}

/// Local selection criteria over public capability metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelSelectionCriteria {
    /// Required input modalities.
    #[serde(default)]
    pub required_input_modalities: Vec<String>,
    /// Required output modalities.
    #[serde(default)]
    pub required_output_modalities: Vec<String>,
    /// Require tool support.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_tools: Option<bool>,
    /// Require structured output support.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_structured_output: Option<bool>,
    /// Require a reasoning-effort parameter.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_reasoning: Option<bool>,
}

/// Reasoning preference used by the catalog helper.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningMode {
    /// Match an effort control.
    Effort,
    /// Match an explicit numeric budget control.
    Budget,
}

/// A protocol-neutral reasoning preference.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReasoningPreference {
    /// Requested control kind.
    pub mode: ReasoningMode,
    /// Exact effort value, when using effort mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<ReasoningEffort>,
    /// Exact budget, when using budget mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<u32>,
}

impl ReasoningPreference {
    /// Creates an effort preference.
    pub fn effort(effort: impl Into<ReasoningEffort>) -> Self {
        Self {
            mode: ReasoningMode::Effort,
            effort: Some(effort.into()),
            budget: None,
        }
    }

    /// Creates a numeric budget preference.
    pub fn budget(budget: u32) -> Self {
        Self {
            mode: ReasoningMode::Budget,
            effort: None,
            budget: Some(budget),
        }
    }
}

fn contains_parameter(item: &ModelCatalogItem, names: &[&str]) -> bool {
    item.supported_parameters
        .as_ref()
        .is_some_and(|parameters| {
            parameters.iter().any(|parameter| {
                names
                    .iter()
                    .any(|name| parameter.eq_ignore_ascii_case(name))
            })
        })
}

fn capability_is_true(value: Option<&CapabilityFlag>) -> bool {
    matches!(value, Some(CapabilityFlag::Bool(true)))
}

fn has_modalities(item: &ModelCatalogItem, required: &[String], input: bool) -> bool {
    if required.is_empty() {
        return true;
    }
    item.architecture.as_ref().is_some_and(|architecture| {
        let available = if input {
            &architecture.input_modalities
        } else {
            &architecture.output_modalities
        };
        required.iter().all(|required| {
            available
                .iter()
                .any(|candidate| candidate.eq_ignore_ascii_case(required))
        })
    })
}

fn price(value: Option<&str>) -> f64 {
    value
        .and_then(|value| value.parse::<f64>().ok())
        .filter(|value| value.is_finite())
        .unwrap_or(f64::MAX)
}

/// Selects and ranks models using only public catalog metadata.
pub fn select_models(
    models: &[ModelCatalogItem],
    criteria: &ModelSelectionCriteria,
) -> Vec<ModelCatalogItem> {
    let mut selected = models
        .iter()
        .filter(|item| {
            has_modalities(item, &criteria.required_input_modalities, true)
                && has_modalities(item, &criteria.required_output_modalities, false)
                && (criteria.require_tools != Some(true)
                    || capability_is_true(
                        item.rainy_capabilities
                            .as_ref()
                            .and_then(|capabilities| capabilities.tools.as_ref()),
                    ))
        })
        .filter(|item| {
            criteria.require_structured_output != Some(true)
                || capability_is_true(
                    item.rainy_capabilities
                        .as_ref()
                        .and_then(|capabilities| capabilities.response_format.as_ref()),
                )
        })
        .filter(|item| {
            criteria.require_reasoning != Some(true)
                || capability_is_true(
                    item.rainy_capabilities
                        .as_ref()
                        .and_then(|capabilities| capabilities.reasoning.as_ref()),
                )
                || contains_parameter(item, &["reasoning", "reasoning_effort"])
        })
        .cloned()
        .collect::<Vec<_>>();

    selected.sort_by(|left, right| {
        let prompt = price(
            left.pricing
                .as_ref()
                .and_then(|pricing| pricing.prompt.as_deref()),
        )
        .partial_cmp(&price(
            right
                .pricing
                .as_ref()
                .and_then(|pricing| pricing.prompt.as_deref()),
        ))
        .unwrap_or(Ordering::Equal);
        prompt.then_with(|| {
            right
                .context_length
                .unwrap_or_default()
                .cmp(&left.context_length.unwrap_or_default())
        })
    });
    selected
}

/// Builds a literal reasoning object from declared public parameter names.
///
/// An effort preference becomes `{ "effort": "..." }` only when the catalog
/// declares `reasoning_effort` or `reasoning.effort`. A budget preference is
/// emitted as `{ "max_tokens": N }` only when the catalog declares a numeric
/// reasoning control. No effort-to-budget conversion is performed.
pub fn build_reasoning_config(
    model: &ModelCatalogItem,
    preference: &ReasoningPreference,
) -> Option<Value> {
    match preference.mode {
        ReasoningMode::Effort => {
            if preference.effort.is_none()
                || !contains_parameter(model, &["reasoning_effort", "reasoning.effort"])
            {
                return None;
            }
            Some(serde_json::to_value(ReasoningConfig::effort(preference.effort.clone()?)).ok()?)
        }
        ReasoningMode::Budget => {
            if preference.budget.is_none()
                || !contains_parameter(model, &["reasoning.max_tokens", "reasoning_max_tokens"])
            {
                return None;
            }
            Some(serde_json::to_value(ReasoningConfig::manual_budget(preference.budget?)).ok()?)
        }
    }
}
