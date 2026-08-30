//! Shared stream response models for OpenAI-compatible endpoints.

use super::Usage;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Usage payload in a Rainy native billing event.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct RainyBillingUsage {
    /// Prompt token count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub prompt_tokens: Option<u32>,
    /// Completion token count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub completion_tokens: Option<u32>,
    /// Reasoning token count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_tokens: Option<u32>,
    /// Image units.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image_units: Option<u32>,
}

/// Rainy native billing event emitted during a Chat stream.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct RainyBillingStreamEvent {
    /// Billing plan identifier.
    #[serde(default, alias = "planId", skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<String>,
    /// Credits charged so far.
    #[serde(
        default,
        alias = "chargedCredits",
        skip_serializing_if = "Option::is_none"
    )]
    pub charged_credits: Option<f64>,
    /// Usage snapshot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<RainyBillingUsage>,
}

/// A typed Chat streaming event.
#[derive(Debug, Clone, PartialEq)]
pub enum ChatStreamEvent {
    /// Standard OpenAI-compatible chunk.
    Chunk(ChatCompletionStreamResponse),
    /// Rainy native billing event.
    Billing(RainyBillingStreamEvent),
    /// A named event not yet classified by this SDK.
    Unknown {
        /// Native SSE event name.
        event: String,
        /// Parsed event payload.
        data: Value,
    },
    /// An unnamed payload that did not match a known chunk.
    Raw(Value),
}

impl ChatStreamEvent {
    /// Classifies an unnamed JSON payload.
    pub fn from_value(value: Value) -> Self {
        if let Ok(chunk) = serde_json::from_value::<ChatCompletionStreamResponse>(value.clone()) {
            return Self::Chunk(chunk);
        }
        if let Ok(billing) = serde_json::from_value::<RainyBillingStreamEvent>(value.clone())
            && (billing.plan_id.is_some()
                || billing.charged_credits.is_some()
                || billing.usage.is_some())
        {
            return Self::Billing(billing);
        }
        Self::Raw(value)
    }

    /// Classifies a payload using the native event name before shape fallback.
    pub fn from_sse_event(event_name: Option<&str>, value: Value) -> Self {
        let Some(event_name) = event_name.filter(|name| !name.is_empty()) else {
            return Self::from_value(value);
        };

        if event_name.eq_ignore_ascii_case("rainy.billing")
            && let Ok(billing) = serde_json::from_value::<RainyBillingStreamEvent>(value.clone())
            && (billing.plan_id.is_some()
                || billing.charged_credits.is_some()
                || billing.usage.is_some())
        {
            return Self::Billing(billing);
        }

        if event_name.eq_ignore_ascii_case("message")
            || event_name.eq_ignore_ascii_case("chat.completion.chunk")
        {
            return match serde_json::from_value::<ChatCompletionStreamResponse>(value.clone()) {
                Ok(chunk) => Self::Chunk(chunk),
                Err(_) => Self::Unknown {
                    event: event_name.to_string(),
                    data: value,
                },
            };
        }

        Self::Unknown {
            event: event_name.to_string(),
            data: value,
        }
    }
}

/// Standard OpenAI-compatible Chat completion chunk.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatCompletionStreamResponse {
    /// Completion identifier.
    pub id: String,
    /// Object label.
    pub object: String,
    /// Creation timestamp.
    pub created: u64,
    /// Model identifier.
    pub model: String,
    /// Stream choices.
    pub choices: Vec<ChatCompletionStreamChoice>,
    /// Optional terminal usage.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
    /// Future-compatible chunk fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// One Chat stream choice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatCompletionStreamChoice {
    /// Choice index.
    pub index: u32,
    /// Incremental delta.
    pub delta: ChatCompletionStreamDelta,
    /// Finish reason, when present.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
    /// Future-compatible choice fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Incremental Chat delta.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChatCompletionStreamDelta {
    /// Role emitted in the first chunk.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// Text delta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// Generic reasoning text field used by some compatible endpoints.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    /// Opaque structured reasoning details.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reasoning_details: Option<Value>,
    /// Legacy generic thought text retained as an opaque protocol field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thought: Option<String>,
    /// Tool call deltas.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
    /// Future-compatible delta fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Incremental tool-call delta.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolCall {
    /// Tool-call index.
    pub index: u32,
    /// Tool-call identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Tool type.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// Function delta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function: Option<ToolCallFunction>,
}

/// Incremental function-call delta.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToolCallFunction {
    /// Function name delta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// JSON argument delta.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}
