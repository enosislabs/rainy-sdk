//! Native Anthropic-compatible Messages models.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Anthropic message content as a text shortcut or content blocks.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AnthropicContent {
    /// Plain text content.
    Text(String),
    /// Typed content blocks.
    Blocks(Vec<AnthropicContentBlock>),
}

impl From<String> for AnthropicContent {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for AnthropicContent {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

/// Anthropic user or assistant message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnthropicMessage {
    /// Message role (`user` or `assistant`).
    pub role: String,
    /// Message content.
    pub content: AnthropicContent,
}

impl AnthropicMessage {
    /// Creates a user message.
    pub fn user(content: impl Into<AnthropicContent>) -> Self {
        Self {
            role: "user".to_string(),
            content: content.into(),
        }
    }

    /// Creates an assistant message.
    pub fn assistant(content: impl Into<AnthropicContent>) -> Self {
        Self {
            role: "assistant".to_string(),
            content: content.into(),
        }
    }
}

/// Anthropic image source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnthropicImageSource {
    /// Source kind, normally `base64` or `url`.
    pub r#type: String,
    /// MIME type for base64 sources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_type: Option<String>,
    /// Base64 data for base64 sources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<String>,
    /// URL for URL sources.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
}

impl AnthropicImageSource {
    /// Creates a base64 image source.
    pub fn base64(media_type: impl Into<String>, data: impl Into<String>) -> Self {
        Self {
            r#type: "base64".to_string(),
            media_type: Some(media_type.into()),
            data: Some(data.into()),
            url: None,
        }
    }

    /// Creates a URL image source.
    pub fn url(url: impl Into<String>) -> Self {
        Self {
            r#type: "url".to_string(),
            media_type: None,
            data: None,
            url: Some(url.into()),
        }
    }
}

/// Native Anthropic content block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicContentBlock {
    /// Text block.
    Text {
        /// Text value.
        text: String,
    },
    /// Image block.
    Image {
        /// Image source.
        source: AnthropicImageSource,
    },
    /// Assistant tool-use block.
    ToolUse {
        /// Tool-use identifier.
        id: String,
        /// Tool name.
        name: String,
        /// Tool input schema value.
        input: Value,
    },
    /// User tool-result block.
    ToolResult {
        /// Tool-use identifier.
        tool_use_id: String,
        /// Result content, which can be a string or block array.
        content: Value,
        /// Whether the tool call failed.
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
    /// Native thinking block preserved for replay.
    Thinking {
        /// Thinking payload.
        thinking: Value,
        /// Provider signature when supplied.
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
}

impl AnthropicContentBlock {
    /// Creates a text block.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// Creates an image block.
    pub fn image(source: AnthropicImageSource) -> Self {
        Self::Image { source }
    }

    /// Creates a tool-use block.
    pub fn tool_use(id: impl Into<String>, name: impl Into<String>, input: Value) -> Self {
        Self::ToolUse {
            id: id.into(),
            name: name.into(),
            input,
        }
    }

    /// Creates a tool-result block.
    pub fn tool_result(
        tool_use_id: impl Into<String>,
        content: Value,
        is_error: Option<bool>,
    ) -> Self {
        Self::ToolResult {
            tool_use_id: tool_use_id.into(),
            content,
            is_error,
        }
    }
}

/// Native Anthropic tool definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnthropicTool {
    /// Tool name.
    pub name: String,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema for tool input.
    pub input_schema: Value,
    /// Future-compatible tool fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl AnthropicTool {
    /// Creates an Anthropic tool definition.
    pub fn new(name: impl Into<String>, input_schema: Value) -> Self {
        Self {
            name: name.into(),
            description: None,
            input_schema,
            extra: BTreeMap::new(),
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// Native Anthropic tool-choice value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AnthropicToolChoice {
    /// A service-defined string mode.
    Mode(String),
    /// Let the model choose.
    Auto {
        /// Choice discriminator.
        r#type: String,
    },
    /// Require one tool call.
    Any {
        /// Choice discriminator.
        r#type: String,
    },
    /// Force one named tool.
    Tool {
        /// Choice discriminator.
        r#type: String,
        /// Tool name.
        name: String,
    },
    /// Preserve a future choice shape.
    Raw(Value),
}

impl AnthropicToolChoice {
    /// Creates `{"type":"auto"}`.
    pub fn auto() -> Self {
        Self::Auto {
            r#type: "auto".to_string(),
        }
    }

    /// Creates a string mode.
    pub fn mode(mode: impl Into<String>) -> Self {
        Self::Mode(mode.into())
    }

    /// Creates `{"type":"any"}`.
    pub fn any() -> Self {
        Self::Any {
            r#type: "any".to_string(),
        }
    }

    /// Forces a named tool.
    pub fn tool(name: impl Into<String>) -> Self {
        Self::Tool {
            r#type: "tool".to_string(),
            name: name.into(),
        }
    }
}

/// Native Anthropic extended-thinking configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnthropicThinking {
    /// Thinking mode, normally `enabled`.
    pub r#type: String,
    /// Explicit token budget required by the native protocol.
    pub budget_tokens: u32,
}

impl AnthropicThinking {
    /// Creates an enabled thinking configuration with an explicit budget.
    pub fn enabled(budget_tokens: u32) -> Self {
        Self {
            r#type: "enabled".to_string(),
            budget_tokens,
        }
    }
}

/// Service tier accepted by compatible Messages endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnthropicServiceTier {
    /// Automatic selection.
    Auto,
    /// Default service.
    Default,
    /// Standard service.
    Standard,
    /// Flexible service.
    Flex,
    /// Scale service.
    Scale,
    /// Priority service.
    Priority,
}

/// Native Anthropic Messages request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnthropicMessageRequest {
    /// Model identifier.
    pub model: String,
    /// User/assistant message history.
    pub messages: Vec<AnthropicMessage>,
    /// Required maximum output token count.
    pub max_tokens: u32,
    /// System prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<AnthropicContent>,
    /// Native metadata value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    /// Stop sequences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    /// Whether to stream native events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Sampling temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Nucleus sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Native tool definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AnthropicTool>>,
    /// Native tool-choice policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<AnthropicToolChoice>,
    /// Native thinking configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<AnthropicThinking>,
    /// Service tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<AnthropicServiceTier>,
    /// Future-compatible native fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl AnthropicMessageRequest {
    /// Creates a Messages request.
    pub fn new(model: impl Into<String>, messages: Vec<AnthropicMessage>, max_tokens: u32) -> Self {
        Self {
            model: model.into(),
            messages,
            max_tokens,
            system: None,
            metadata: None,
            stop_sequences: None,
            stream: None,
            temperature: None,
            top_p: None,
            tools: None,
            tool_choice: None,
            thinking: None,
            service_tier: None,
            extra: BTreeMap::new(),
        }
    }

    /// Sets stream mode.
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Sets temperature, clamped to the protocol range.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature.clamp(0.0, 2.0));
        self
    }

    /// Sets top-p, clamped to the protocol range.
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p.clamp(0.0, 1.0));
        self
    }

    /// Sets stop sequences.
    pub fn with_stop_sequences(mut self, values: Vec<String>) -> Self {
        self.stop_sequences = Some(values);
        self
    }

    /// Sets a system prompt.
    pub fn with_system(mut self, system: impl Into<AnthropicContent>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Sets native tools.
    pub fn with_tools(mut self, tools: Vec<AnthropicTool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Sets native tool choice.
    pub fn with_tool_choice(mut self, choice: AnthropicToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    /// Sets native thinking with an explicit budget.
    pub fn with_thinking(mut self, budget_tokens: u32) -> Self {
        self.thinking = Some(AnthropicThinking::enabled(budget_tokens));
        self
    }

    /// Sets the native thinking configuration directly.
    pub fn with_thinking_config(mut self, thinking: AnthropicThinking) -> Self {
        self.thinking = Some(thinking);
        self
    }

    /// Sets service tier.
    pub fn with_service_tier(mut self, tier: AnthropicServiceTier) -> Self {
        self.service_tier = Some(tier);
        self
    }

    /// Sets native metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Adds a future-compatible field.
    pub fn with_extra(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }

    /// Validates required native request constraints.
    pub fn validate(&self) -> Result<(), String> {
        if self.max_tokens == 0 {
            return Err("max_tokens must be greater than zero".to_string());
        }
        if self
            .thinking
            .as_ref()
            .is_some_and(|value| value.budget_tokens == 0)
        {
            return Err("thinking.budget_tokens must be greater than zero".to_string());
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
        Ok(())
    }
}

/// Native Anthropic usage object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AnthropicUsage {
    /// Input tokens.
    #[serde(default)]
    pub input_tokens: u32,
    /// Output tokens.
    #[serde(default)]
    pub output_tokens: u32,
    /// Cache creation tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
    /// Cache read tokens.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
    /// Future usage fields.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

/// Native Anthropic Messages response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnthropicMessageResponse {
    /// Response identifier.
    pub id: String,
    /// Object type, normally `message`.
    pub r#type: String,
    /// Assistant role.
    pub role: String,
    /// Output content blocks.
    pub content: Vec<AnthropicContentBlock>,
    /// Model identifier.
    pub model: String,
    /// Stop reason.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    /// Stop sequence.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    /// Token usage.
    pub usage: AnthropicUsage,
    /// Future response fields.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

impl AnthropicMessageResponse {
    /// Concatenates text blocks while preserving all original blocks in `content`.
    pub fn text(&self) -> String {
        self.content
            .iter()
            .filter_map(|block| match block {
                AnthropicContentBlock::Text { text } => Some(text.as_str()),
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("")
    }
}

/// Native Anthropic stream event with its SSE name and unmodified JSON data.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AnthropicMessageStreamEvent {
    /// Native SSE event name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    /// Classified event category.
    #[serde(skip)]
    pub kind: AnthropicMessageStreamEventType,
    /// Parsed event payload.
    pub data: Value,
}

impl<'de> Deserialize<'de> for AnthropicMessageStreamEvent {
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

/// Categories of native Anthropic stream events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnthropicMessageStreamEventType {
    /// Initial message metadata.
    MessageStart,
    /// Content block started.
    ContentBlockStart,
    /// Content delta.
    ContentBlockDelta,
    /// Content block ended.
    ContentBlockStop,
    /// Message usage/stop delta.
    MessageDelta,
    /// Message completed.
    MessageStop,
    /// Keepalive ping.
    Ping,
    /// Native error event.
    Error,
    /// Future event.
    #[default]
    Unknown,
}

impl AnthropicMessageStreamEventType {
    /// Classifies a native event name.
    pub fn from_name(name: Option<&str>) -> Self {
        match name.unwrap_or_default() {
            "message_start" => Self::MessageStart,
            "content_block_start" => Self::ContentBlockStart,
            "content_block_delta" => Self::ContentBlockDelta,
            "content_block_stop" => Self::ContentBlockStop,
            "message_delta" => Self::MessageDelta,
            "message_stop" => Self::MessageStop,
            "ping" => Self::Ping,
            "error" => Self::Error,
            _ => Self::Unknown,
        }
    }
}

impl AnthropicMessageStreamEvent {
    /// Creates an event while preserving its payload.
    pub fn new(event: Option<String>, data: Value) -> Self {
        let kind = match event.as_deref() {
            Some(name) => AnthropicMessageStreamEventType::from_name(Some(name)),
            None => {
                AnthropicMessageStreamEventType::from_name(data.get("type").and_then(Value::as_str))
            }
        };
        Self { event, kind, data }
    }
}
