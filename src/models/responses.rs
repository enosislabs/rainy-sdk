//! Responses-specific typed helpers.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

/// Typed input item commonly used for Responses multimodal requests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponsesInputItem {
    /// Plain text input item.
    InputText {
        /// Text value.
        text: String,
    },
    /// Image input item.
    InputImage {
        /// Image URL or data URL.
        image_url: String,
        /// Optional detail hint.
        #[serde(skip_serializing_if = "Option::is_none")]
        detail: Option<String>,
    },
    /// Function-call output continuation item.
    FunctionCallOutput {
        /// Function call identifier.
        call_id: String,
        /// Tool output.
        output: Value,
    },
}

/// Meaningful Responses stream event categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResponsesEventType {
    /// A response object was created.
    ResponseCreated,
    /// A response entered processing.
    ResponseInProgress,
    /// A response was queued.
    ResponseQueued,
    /// A response object completed.
    ResponseCompleted,
    /// A response failed or became incomplete.
    ResponseFailed,
    /// An output item was added.
    OutputItemAdded,
    /// An output item completed.
    OutputItemDone,
    /// An output content part was added.
    ContentPartAdded,
    /// An output content part completed.
    ContentPartDone,
    /// Text delta was emitted.
    OutputTextDelta,
    /// Text output completed.
    OutputTextDone,
    /// Function-call arguments delta was emitted.
    FunctionCallArgumentsDelta,
    /// Function-call arguments completed.
    FunctionCallArgumentsDone,
    /// A reasoning summary part was added.
    ReasoningSummaryPartAdded,
    /// A reasoning summary text delta was emitted.
    ReasoningSummaryTextDelta,
    /// A reasoning summary part completed.
    ReasoningSummaryPartDone,
    /// A web-search call changed state.
    WebSearchCall,
    /// The stream reported an error event.
    Error,
    /// A server-defined event not yet known to this SDK.
    #[default]
    Unknown,
}

impl ResponsesEventType {
    /// Classifies a native Responses event name.
    pub fn from_name(name: Option<&str>) -> Self {
        match name.unwrap_or_default().to_ascii_lowercase().as_str() {
            "response.created" => Self::ResponseCreated,
            "response.in_progress" => Self::ResponseInProgress,
            "response.queued" => Self::ResponseQueued,
            "response.completed" => Self::ResponseCompleted,
            "response.failed" => Self::ResponseFailed,
            "response.incomplete" => Self::ResponseFailed,
            "response.output_item.added" => Self::OutputItemAdded,
            "response.output_item.done" => Self::OutputItemDone,
            "response.content_part.added" => Self::ContentPartAdded,
            "response.content_part.done" => Self::ContentPartDone,
            "response.output_text.delta" => Self::OutputTextDelta,
            "response.output_text.done" => Self::OutputTextDone,
            "response.function_call_arguments.delta" => Self::FunctionCallArgumentsDelta,
            "response.function_call_arguments.done" => Self::FunctionCallArgumentsDone,
            "response.reasoning_summary_part.added" => Self::ReasoningSummaryPartAdded,
            "response.reasoning_summary_text.delta" => Self::ReasoningSummaryTextDelta,
            "response.reasoning_summary_part.done" => Self::ReasoningSummaryPartDone,
            name if name.starts_with("response.web_search_call.") => Self::WebSearchCall,
            "error" => Self::Error,
            _ => Self::Unknown,
        }
    }
}

/// A Responses SSE event with native name, category, and unmodified payload.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ResponsesEvent {
    /// Native SSE event name, when supplied by the server or payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    /// Event category derived from `event` or payload `type`.
    #[serde(skip)]
    pub kind: ResponsesEventType,
    /// Parsed event payload.
    pub data: Value,
}

impl<'de> Deserialize<'de> for ResponsesEvent {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct WireEvent {
            #[serde(default)]
            event: Option<String>,
            data: Value,
        }

        let event = WireEvent::deserialize(deserializer)?;
        Ok(Self::new(event.event, event.data))
    }
}

impl ResponsesEvent {
    /// Creates an event and classifies it without modifying the payload.
    pub fn new(event: Option<String>, data: Value) -> Self {
        let event_name = event
            .as_deref()
            .or_else(|| data.get("type").and_then(Value::as_str));
        Self {
            kind: ResponsesEventType::from_name(event_name),
            event,
            data,
        }
    }

    /// Returns the text delta when this event carries one.
    pub fn text_delta(&self) -> Option<&str> {
        self.data
            .get("delta")
            .and_then(Value::as_str)
            .or_else(|| self.data.get("text").and_then(Value::as_str))
    }
}
