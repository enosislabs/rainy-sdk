//! OpenAI-compatible Responses models and stream event helpers.

use super::{ReasoningEffort, ReasoningRequest};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

/// Common typed input items for multimodal Responses requests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponsesInputItem {
    /// Text input item.
    InputText {
        /// Text content.
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
    /// Function-call output used for a follow-up turn.
    FunctionCallOutput {
        /// Function call identifier.
        call_id: String,
        /// Tool output value.
        output: Value,
    },
}

/// Known Responses stream event categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ResponsesEventType {
    /// Response created.
    ResponseCreated,
    /// Response is in progress.
    ResponseInProgress,
    /// Response queued.
    ResponseQueued,
    /// Response completed.
    ResponseCompleted,
    /// Response failed or became incomplete.
    ResponseFailed,
    /// Output item added.
    OutputItemAdded,
    /// Output item completed.
    OutputItemDone,
    /// Content part added.
    ContentPartAdded,
    /// Content part completed.
    ContentPartDone,
    /// Output text delta.
    OutputTextDelta,
    /// Output text completed.
    OutputTextDone,
    /// Function-call arguments delta.
    FunctionCallArgumentsDelta,
    /// Function-call arguments completed.
    FunctionCallArgumentsDone,
    /// Reasoning summary part added.
    ReasoningSummaryPartAdded,
    /// Reasoning summary text delta.
    ReasoningSummaryTextDelta,
    /// Reasoning summary part completed.
    ReasoningSummaryPartDone,
    /// Web-search call event.
    WebSearchCall,
    /// Error event.
    Error,
    /// An event not yet known to this SDK.
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
            "response.failed" | "response.incomplete" => Self::ResponseFailed,
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
            value if value.starts_with("response.web_search_call.") => Self::WebSearchCall,
            "error" => Self::Error,
            _ => Self::Unknown,
        }
    }
}

/// A Responses SSE event retaining its native event name and payload.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct ResponsesEvent {
    /// Native SSE event name, when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    /// Classified event category.
    #[serde(skip)]
    pub kind: ResponsesEventType,
    /// Unmodified parsed JSON payload.
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
        let value = WireEvent::deserialize(deserializer)?;
        Ok(Self::new(value.event, value.data))
    }
}

impl ResponsesEvent {
    /// Creates a classified event without rewriting its payload.
    pub fn new(event: Option<String>, data: Value) -> Self {
        let name = event
            .as_deref()
            .or_else(|| data.get("type").and_then(Value::as_str));
        Self {
            kind: ResponsesEventType::from_name(name),
            event,
            data,
        }
    }

    /// Returns a text delta from common Responses event shapes.
    pub fn text_delta(&self) -> Option<&str> {
        self.data
            .get("delta")
            .and_then(Value::as_str)
            .or_else(|| self.data.get("text").and_then(Value::as_str))
    }
}

/// OpenAI-compatible Responses request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ResponsesRequest {
    /// Model identifier.
    pub model: String,
    /// String, input items, or arbitrary compatible input.
    pub input: Value,
    /// Whether to stream events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Responses tool definitions, intentionally open for built-in tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Value>>,
    /// Tool selection object or string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
    /// Structured output format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<Value>,
    /// Sampling temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Nucleus sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Maximum output tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,
    /// Maximum hosted-tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tool_calls: Option<u32>,
    /// End-user identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Prompt cache key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    /// Prompt cache options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_options: Option<Value>,
    /// Structured or boolean reasoning control.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningRequest>,
    /// First-class effort label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Include reasoning where supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_reasoning: Option<bool>,
    /// Whether parallel tool calls are allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// Stream options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<Value>,
    /// String metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Service tier hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    /// Whether the response is stored.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    /// Stable safety identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,
    /// Provider extension options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_options: Option<Value>,
    /// Prompt-cache retention hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<String>,
    /// Text/output configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<Value>,
    /// System-level instructions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instructions: Option<String>,
    /// Include directives.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include: Option<Vec<String>>,
    /// Previous response identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub previous_response_id: Option<String>,
    /// Conversation identifier/object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversation: Option<Value>,
    /// Hosted prompt object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<Value>,
    /// Background processing flag.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub background: Option<bool>,
    /// Context-management directives.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_management: Option<Vec<Value>>,
    /// Truncation mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub truncation: Option<String>,
    /// Future-compatible fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl ResponsesRequest {
    /// Creates a request from arbitrary Responses input.
    pub fn new(model: impl Into<String>, input: Value) -> Self {
        Self {
            model: model.into(),
            input,
            stream: None,
            tools: None,
            tool_choice: None,
            response_format: None,
            temperature: None,
            top_p: None,
            max_output_tokens: None,
            max_tool_calls: None,
            user: None,
            prompt_cache_key: None,
            prompt_cache_options: None,
            reasoning: None,
            reasoning_effort: None,
            include_reasoning: None,
            parallel_tool_calls: None,
            stream_options: None,
            metadata: None,
            service_tier: None,
            store: None,
            safety_identifier: None,
            provider_options: None,
            prompt_cache_retention: None,
            text: None,
            instructions: None,
            include: None,
            previous_response_id: None,
            conversation: None,
            prompt: None,
            background: None,
            context_management: None,
            truncation: None,
            extra: BTreeMap::new(),
        }
    }

    /// Creates a plain-text input request.
    pub fn text(model: impl Into<String>, input: impl Into<String>) -> Self {
        Self::new(model, Value::String(input.into()))
    }

    /// Sets streaming mode.
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Sets a boolean, structured, or raw reasoning value.
    pub fn with_reasoning(mut self, reasoning: impl Into<ReasoningRequest>) -> Self {
        self.reasoning = Some(reasoning.into());
        self
    }

    /// Sets an exact effort label as the top-level `reasoning_effort` field.
    pub fn with_reasoning_effort(mut self, effort: impl Into<ReasoningEffort>) -> Self {
        self.reasoning_effort = Some(effort.into());
        self
    }

    /// Sets an explicit numeric budget as `reasoning.max_tokens`.
    pub fn with_reasoning_budget(mut self, max_tokens: u32) -> Self {
        self.reasoning = Some(super::ReasoningConfig::manual_budget(max_tokens).into());
        self
    }

    /// Requests reasoning content where supported.
    pub fn with_include_reasoning(mut self, include: bool) -> Self {
        self.include_reasoning = Some(include);
        self
    }

    /// Sets the maximum output token count.
    pub fn with_max_output_tokens(mut self, value: u32) -> Self {
        self.max_output_tokens = Some(value);
        self
    }

    /// Sets the maximum number of hosted-tool calls.
    pub fn with_max_tool_calls(mut self, value: u32) -> Self {
        self.max_tool_calls = Some(value);
        self
    }

    /// Sets a structured output format.
    pub fn with_response_format(mut self, value: Value) -> Self {
        self.response_format = Some(value);
        self
    }

    /// Sets the Responses text/output configuration.
    pub fn with_text(mut self, value: Value) -> Self {
        self.text = Some(value);
        self
    }

    /// Sets the user identifier.
    pub fn with_user(mut self, value: impl Into<String>) -> Self {
        self.user = Some(value.into());
        self
    }

    /// Sets a prompt-cache key.
    pub fn with_prompt_cache_key(mut self, value: impl Into<String>) -> Self {
        self.prompt_cache_key = Some(value.into());
        self
    }

    /// Sets prompt-cache options.
    pub fn with_prompt_cache_options(mut self, value: Value) -> Self {
        self.prompt_cache_options = Some(value);
        self
    }

    /// Sets provider extension options.
    pub fn with_provider_options(mut self, value: Value) -> Self {
        self.provider_options = Some(value);
        self
    }

    /// Sets instructions.
    pub fn with_instructions(mut self, value: impl Into<String>) -> Self {
        self.instructions = Some(value.into());
        self
    }

    /// Sets previous response identifier.
    pub fn with_previous_response_id(mut self, value: impl Into<String>) -> Self {
        self.previous_response_id = Some(value.into());
        self
    }

    /// Sets tool choice.
    pub fn with_tool_choice(mut self, value: impl Into<Value>) -> Self {
        self.tool_choice = Some(value.into());
        self
    }

    /// Sets parallel tool-call behavior.
    pub fn with_parallel_tool_calls(mut self, value: bool) -> Self {
        self.parallel_tool_calls = Some(value);
        self
    }

    /// Sets stream options.
    pub fn with_stream_options(mut self, value: Value) -> Self {
        self.stream_options = Some(value);
        self
    }

    /// Sets whether the response is stored.
    pub fn with_store(mut self, value: bool) -> Self {
        self.store = Some(value);
        self
    }

    /// Sets background processing.
    pub fn with_background(mut self, value: bool) -> Self {
        self.background = Some(value);
        self
    }

    /// Sets include directives.
    pub fn with_include(mut self, value: Vec<String>) -> Self {
        self.include = Some(value);
        self
    }

    /// Sets service tier.
    pub fn with_service_tier(mut self, value: impl Into<String>) -> Self {
        self.service_tier = Some(value.into());
        self
    }

    /// Sets a stable safety identifier.
    pub fn with_safety_identifier(mut self, value: impl Into<String>) -> Self {
        self.safety_identifier = Some(value.into());
        self
    }

    /// Sets string metadata.
    pub fn with_metadata(mut self, value: HashMap<String, String>) -> Self {
        self.metadata = Some(value);
        self
    }

    /// Replaces the open tool list.
    pub fn with_tools(mut self, value: Vec<Value>) -> Self {
        self.tools = Some(value);
        self
    }

    /// Adds a Responses function tool.
    pub fn add_function_tool(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Value,
    ) -> Self {
        self.tools
            .get_or_insert_with(Vec::new)
            .push(serde_json::json!({
                "type": "function",
                "name": name.into(),
                "description": description.into(),
                "parameters": parameters
            }));
        self
    }

    /// Adds a strict Responses function tool.
    pub fn add_strict_function_tool(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Value,
    ) -> Self {
        self.tools
            .get_or_insert_with(Vec::new)
            .push(serde_json::json!({
                "type": "function",
                "name": name.into(),
                "description": description.into(),
                "parameters": parameters,
                "strict": true
            }));
        self
    }

    /// Adds a hosted web-search tool.
    pub fn add_web_search_tool(mut self) -> Self {
        self.tools
            .get_or_insert_with(Vec::new)
            .push(serde_json::json!({"type": "web_search"}));
        self
    }

    /// Adds a hosted file-search tool.
    pub fn add_file_search_tool<I, S>(mut self, ids: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.tools
            .get_or_insert_with(Vec::new)
            .push(serde_json::json!({
                "type": "file_search",
                "vector_store_ids": ids.into_iter().map(Into::into).collect::<Vec<String>>()
            }));
        self
    }

    /// Adds an arbitrary open tool definition.
    pub fn add_tool(mut self, tool: Value) -> Self {
        self.tools.get_or_insert_with(Vec::new).push(tool);
        self
    }

    /// Creates a function-call output input item.
    pub fn function_call_output(call_id: impl Into<String>, output: impl Into<Value>) -> Value {
        serde_json::json!({
            "type": "function_call_output",
            "call_id": call_id.into(),
            "output": output.into()
        })
    }

    /// Adds a future-compatible request field.
    pub fn with_extra(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }

    /// Validates protocol-level request constraints without inspecting the model name.
    pub fn validate(&self) -> Result<(), String> {
        if self.max_output_tokens == Some(0) || self.max_tool_calls == Some(0) {
            return Err("token and tool-call limits must be greater than zero".to_string());
        }
        if let Some(value) = self.temperature
            && (!value.is_finite() || !(0.0..=2.0).contains(&value))
        {
            return Err("temperature must be between 0.0 and 2.0".to_string());
        }
        if let Some(value) = self.top_p
            && (!value.is_finite() || !(0.0..=1.0).contains(&value))
        {
            return Err("top_p must be between 0.0 and 1.0".to_string());
        }
        if let Some(ReasoningRequest::Config(config)) = &self.reasoning {
            config.validate()?;
        }
        Ok(())
    }
}

/// Responses token usage.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResponsesUsage {
    /// Input tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u32>,
    /// Output tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u32>,
    /// Cache creation tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
    /// Cache read tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
    /// Future usage fields.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

/// OpenAI-compatible Responses response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ResponsesApiResponse {
    /// Response identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Object label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// Model identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Plain text shortcut.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_text: Option<String>,
    /// Structured output items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Vec<Value>>,
    /// Response status.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// Error payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Value>,
    /// Incomplete details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub incomplete_details: Option<Value>,
    /// Token usage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ResponsesUsage>,
    /// Future response fields.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

impl ResponsesApiResponse {
    /// Returns function-call output items.
    pub fn function_calls(&self) -> Vec<&Value> {
        self.output
            .iter()
            .flatten()
            .filter(|item| item.get("type").and_then(Value::as_str) == Some("function_call"))
            .collect()
    }

    /// Returns concatenated output text from direct or nested output fields.
    pub fn text(&self) -> Option<String> {
        if let Some(text) = &self.output_text {
            return Some(text.clone());
        }
        let parts: Vec<&str> = self
            .output
            .iter()
            .flatten()
            .filter_map(|item| item.get("content"))
            .filter_map(Value::as_array)
            .flatten()
            .filter(|content| content.get("type").and_then(Value::as_str) == Some("output_text"))
            .filter_map(|content| content.get("text"))
            .filter_map(Value::as_str)
            .collect();
        (!parts.is_empty()).then(|| parts.join(""))
    }
}

/// Stream event returned by [`crate::RainyClient::create_response_stream`].
pub type ResponsesStreamEvent = ResponsesEvent;
