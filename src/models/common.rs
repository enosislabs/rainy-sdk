//! Shared models used by more than one public protocol.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// Token usage for a chat completion.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Usage {
    /// Number of input/prompt tokens.
    #[serde(default, alias = "input_tokens")]
    pub prompt_tokens: u32,
    /// Number of generated/output tokens.
    #[serde(default, alias = "output_tokens")]
    pub completion_tokens: u32,
    /// Total tokens consumed.
    #[serde(default)]
    pub total_tokens: u32,
}

/// Metadata extracted from response headers.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RequestMetadata {
    /// Elapsed request time in milliseconds.
    pub response_time: Option<u64>,
    /// Opaque provider label returned by a compatible gateway, when supplied.
    pub provider: Option<String>,
    /// Number of tokens reported by the gateway.
    pub tokens_used: Option<u32>,
    /// Credits charged by Rainy, when supplied.
    pub credits_used: Option<f64>,
    /// Credits remaining after the request, when supplied.
    pub credits_remaining: Option<f64>,
    /// Opaque request correlation identifier.
    pub request_id: Option<String>,
    /// Count of compatibility warnings returned by Rainy.
    pub compat_warnings: Option<u32>,
    /// Response mode selected by Rainy (`raw` or `envelope`).
    pub response_mode: Option<String>,
    /// Rainy billing plan label, when supplied.
    pub billing_plan: Option<String>,
    /// Rainy credits charged, when supplied.
    pub rainy_credits_charged: Option<f64>,
    /// Rainy daily credit balance, when supplied.
    pub rainy_daily_credits_remaining: Option<String>,
    /// Sanitized parameter summary returned by Rainy.
    pub rainy_sanitized_params: Option<String>,
    /// Rainy billing adjustment status, when supplied.
    pub rainy_billing_adjustment: Option<String>,
    /// Rainy billing amount that remains outstanding, when supplied.
    pub rainy_billing_outstanding_credits: Option<f64>,
}

/// A standard Rainy success envelope.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RainyEnvelope<T> {
    /// Whether the operation succeeded.
    pub success: bool,
    /// Typed response payload.
    pub data: T,
    /// Optional Rainy-only metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<RainyEnvelopeMeta>,
}

/// Additive Rainy envelope metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RainyEnvelopeMeta {
    /// Billing plan label.
    #[serde(
        rename = "billingPlan",
        alias = "billing_plan",
        skip_serializing_if = "Option::is_none"
    )]
    pub billing_plan: Option<String>,
    /// Credits charged.
    #[serde(
        rename = "creditsCharged",
        alias = "credits_charged",
        skip_serializing_if = "Option::is_none"
    )]
    pub credits_charged: Option<f64>,
    /// Markup percentage, when returned by Rainy.
    #[serde(
        rename = "markupPercent",
        alias = "markup_percent",
        skip_serializing_if = "Option::is_none"
    )]
    pub markup_percent: Option<f64>,
    /// Daily credits remaining.
    #[serde(
        rename = "dailyCreditsRemaining",
        alias = "daily_credits_remaining",
        skip_serializing_if = "Option::is_none"
    )]
    pub daily_credits_remaining: Option<String>,
    /// Non-blocking compatibility warnings.
    #[serde(
        rename = "compatWarnings",
        alias = "compat_warnings",
        skip_serializing_if = "Option::is_none"
    )]
    pub compat_warnings: Option<Vec<CompatWarning>>,
    /// Features used by the request.
    #[serde(
        rename = "featuresUsed",
        alias = "features_used",
        skip_serializing_if = "Option::is_none"
    )]
    pub features_used: Option<FeaturesUsed>,
    /// Reasoning metadata when the gateway reports it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningMeta>,
    /// Future envelope fields.
    #[serde(flatten, default)]
    pub extra: HashMap<String, Value>,
}

/// A non-blocking compatibility warning.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompatWarning {
    /// Stable warning code.
    pub code: String,
    /// Human-readable warning.
    pub message: String,
    /// JSON path associated with the warning, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Feature summary returned by Rainy envelope mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FeaturesUsed {
    /// Whether reasoning was used.
    pub reasoning: bool,
    /// Whether image input was used.
    #[serde(rename = "imageInput")]
    pub image_input: bool,
    /// Whether tools were used.
    pub tools: bool,
    /// Whether structured output was requested.
    #[serde(rename = "structuredOutput")]
    pub structured_output: bool,
}

/// Reasoning summary metadata returned by Rainy envelope mode.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReasoningMeta {
    /// Whether reasoning was present.
    pub present: bool,
    /// Whether a summary was present.
    pub summary_present: bool,
    /// Reasoning token count, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<u32>,
}

/// Basic health response for a Rainy-compatible service.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HealthStatus {
    /// Overall service status.
    pub status: String,
    /// Timestamp supplied by the service.
    pub timestamp: String,
    /// Uptime when supplied; zero when the public health route omits it.
    pub uptime: f64,
    /// Service health flags.
    pub services: ServiceStatus,
}

/// Health flags for commonly reported service dependencies.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ServiceStatus {
    /// Database health, when known.
    pub database: bool,
    /// Redis health, when reported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis: Option<bool>,
    /// Inference service health, when known.
    pub providers: bool,
}

/// Compatibility summary derived from a public model-list response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AvailableModels {
    /// Model identifiers grouped by the optional `owner/model` convention.
    #[serde(default)]
    pub providers: HashMap<String, Vec<String>>,
    /// Total number of model identifiers.
    #[serde(default)]
    pub total_models: usize,
    /// Sorted owner labels derived from model identifiers.
    #[serde(default)]
    pub active_providers: Vec<String>,
}

/// Credit information returned by older compatibility helpers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CreditInfo {
    /// Balance before the request.
    pub current_credits: f64,
    /// Estimated request cost.
    pub estimated_cost: f64,
    /// Balance after the request.
    pub credits_after_request: f64,
    /// Reset date, when supplied.
    pub reset_date: String,
}

/// Search provider selector used by the optional Rainy search extension.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResearchProvider {
    /// Let the service use its semantic search provider.
    #[default]
    Exa,
    /// Use the service's comprehensive search provider.
    Tavily,
    /// Let the service choose.
    Auto,
}

/// Search depth selector used by the optional Rainy search extension.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResearchDepth {
    /// Faster search with a smaller result set.
    #[default]
    Basic,
    /// More thorough search.
    Advanced,
}
