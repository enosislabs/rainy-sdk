//! Anthropic-compatible Messages request, response, and stream types.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Anthropic content, either a text string or typed content blocks.
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

/// Anthropic message role/content pair.
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

/// Anthropic image source descriptor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnthropicImageSource {
    /// Source kind, normally `base64` or `url`.
    pub r#type: String,
    /// Media type for base64 sources.
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

/// Typed Anthropic message content block.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicContentBlock {
    /// Text content block.
    Text {
        /// Text value.
        text: String,
    },
    /// Image content block.
    Image {
        /// Image source.
        source: AnthropicImageSource,
    },
    /// Tool-use block emitted by the assistant.
    ToolUse {
        /// Tool-use identifier.
        id: String,
        /// Tool name.
        name: String,
        /// Tool input.
        input: Value,
    },
    /// Tool-result block supplied by the caller.
    ToolResult {
        /// Tool-use identifier.
        tool_use_id: String,
        /// Result content.
        content: Value,
        /// Whether the tool failed.
        #[serde(skip_serializing_if = "Option::is_none")]
        is_error: Option<bool>,
    },
    /// Provider reasoning/thinking block preserved for continuation.
    Thinking {
        /// Thinking text or provider payload.
        thinking: Value,
        /// Signature supplied by the provider, when present.
        #[serde(skip_serializing_if = "Option::is_none")]
        signature: Option<String>,
    },
}

impl AnthropicContentBlock {
    /// Creates a text content block.
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// Creates an image content block.
    pub fn image(source: AnthropicImageSource) -> Self {
        Self::Image { source }
    }

    /// Creates a tool-use block for assistant message replay.
    pub fn tool_use(id: impl Into<String>, name: impl Into<String>, input: Value) -> Self {
        Self::ToolUse {
            id: id.into(),
            name: name.into(),
            input,
        }
    }

    /// Creates a tool-result block for user message replay.
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

/// Anthropic tool definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnthropicTool {
    /// Tool name.
    pub name: String,
    /// Human-readable tool description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON schema for tool input.
    pub input_schema: Value,
    /// Explicit future fields.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

/// Anthropic tool choice value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum AnthropicToolChoice {
    /// A provider-defined string mode, such as `auto`.
    Mode(String),
    /// Let the model decide.
    Auto {
        /// Tool-choice discriminator, normally `auto`.
        r#type: String,
    },
    /// Require a tool call.
    Any {
        /// Tool-choice discriminator, normally `any`.
        r#type: String,
    },
    /// Force one named tool.
    Tool {
        /// Tool-choice discriminator, normally `tool`.
        r#type: String,
        /// Name of the forced tool.
        name: String,
    },
    /// A future tool-choice object retained verbatim.
    Raw(Value),
}

impl AnthropicToolChoice {
    /// Creates an automatic tool-choice value.
    pub fn auto() -> Self {
        Self::Auto {
            r#type: "auto".to_string(),
        }
    }

    /// Creates a string-valued tool-choice mode.
    pub fn mode(mode: impl Into<String>) -> Self {
        Self::Mode(mode.into())
    }

    /// Creates a required tool-choice value.
    pub fn any() -> Self {
        Self::Any {
            r#type: "any".to_string(),
        }
    }

    /// Creates a forced named tool-choice value.
    pub fn tool(name: impl Into<String>) -> Self {
        Self::Tool {
            r#type: "tool".to_string(),
            name: name.into(),
        }
    }
}

/// Anthropic extended-thinking configuration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AnthropicThinking {
    /// Thinking mode, normally `enabled`.
    pub r#type: String,
    /// Explicit token budget required by Anthropic extended thinking.
    pub budget_tokens: u32,
}

impl AnthropicThinking {
    /// Enables Anthropic thinking with an explicit budget.
    pub fn enabled(budget_tokens: u32) -> Self {
        Self {
            r#type: "enabled".to_string(),
            budget_tokens,
        }
    }
}

/// Service tier accepted by Rainy's Messages route.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AnthropicServiceTier {
    /// Automatic routing.
    Auto,
    /// Default service.
    Default,
    /// Anthropic-standard service.
    Standard,
    /// Flexible service.
    Flex,
    /// Scale service.
    Scale,
    /// Priority service.
    Priority,
}

/// Request body for `POST /api/v1/messages`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AnthropicMessageRequest {
    /// Model identifier.
    pub model: String,
    /// Conversation messages.
    pub messages: Vec<AnthropicMessage>,
    /// Maximum output tokens; required by Rainy.
    pub max_tokens: u32,
    /// Optional system prompt.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub system: Option<AnthropicContent>,
    /// Request metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<Value>,
    /// Stop sequences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequences: Option<Vec<String>>,
    /// Request streaming.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Sampling temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Nucleus sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Tools supplied to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<AnthropicTool>>,
    /// Tool choice policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<AnthropicToolChoice>,
    /// Anthropic extended-thinking control.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<AnthropicThinking>,
    /// Requested service tier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<AnthropicServiceTier>,
    /// Explicit future request fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl AnthropicMessageRequest {
    /// Creates a Messages request with the required model, messages, and output budget.
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

    /// Sets streaming mode.
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Sets the sampling temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature.clamp(0.0, 2.0));
        self
    }

    /// Sets nucleus sampling.
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p.clamp(0.0, 1.0));
        self
    }

    /// Sets stop sequences.
    pub fn with_stop_sequences(mut self, stop_sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(stop_sequences);
        self
    }

    /// Sets the available Anthropic tools.
    pub fn with_tools(mut self, tools: Vec<AnthropicTool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Sets the tool-choice policy.
    pub fn with_tool_choice(mut self, tool_choice: AnthropicToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets Anthropic extended thinking with a numeric budget.
    pub fn with_thinking(mut self, budget_tokens: u32) -> Self {
        self.thinking = Some(AnthropicThinking::enabled(budget_tokens));
        self
    }

    /// Sets the Messages service tier.
    pub fn with_service_tier(mut self, service_tier: AnthropicServiceTier) -> Self {
        self.service_tier = Some(service_tier);
        self
    }

    /// Sets request metadata.
    pub fn with_metadata(mut self, metadata: Value) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Sets a system prompt.
    pub fn with_system(mut self, system: impl Into<AnthropicContent>) -> Self {
        self.system = Some(system.into());
        self
    }

    /// Adds an explicit future request field.
    pub fn with_extra(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }
}

/// Anthropic usage object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct AnthropicUsage {
    /// Input token count.
    #[serde(default)]
    pub input_tokens: u32,
    /// Output token count.
    #[serde(default)]
    pub output_tokens: u32,
    /// Cache-creation input token count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
    /// Cache-read input token count.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
    /// Future usage fields.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

/// Anthropic Messages response body.
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
    /// Model used.
    pub model: String,
    /// Stop reason, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_reason: Option<String>,
    /// Stop sequence, when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    /// Token usage.
    pub usage: AnthropicUsage,
    /// Future response fields.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

impl AnthropicMessageResponse {
    /// Concatenates text blocks from the response in display order.
    ///
    /// Tool-use, image, and thinking blocks are preserved in [`Self::content`]
    /// but are not included in this convenience string.
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

/// Typed event from the Anthropic Messages SSE stream.
#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct AnthropicMessageStreamEvent {
    /// Native SSE event name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub event: Option<String>,
    /// Typed native event category.
    #[serde(skip)]
    pub kind: AnthropicMessageStreamEventType,
    /// Event JSON payload.
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

        let event = WireEvent::deserialize(deserializer)?;
        Ok(Self::new(event.event, event.data))
    }
}

/// Native event names emitted by the Anthropic Messages stream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AnthropicMessageStreamEventType {
    /// Initial message metadata.
    MessageStart,
    /// A new content block began.
    ContentBlockStart,
    /// A content block delta was emitted.
    ContentBlockDelta,
    /// A content block ended.
    ContentBlockStop,
    /// Message-level usage/stop metadata changed.
    MessageDelta,
    /// The message stream completed.
    MessageStop,
    /// Keepalive event.
    Ping,
    /// Provider/API error event.
    Error,
    /// A future or provider-specific event.
    #[default]
    Unknown,
}

impl AnthropicMessageStreamEventType {
    /// Classifies an Anthropic native SSE event name.
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
    /// Creates a typed event while retaining its unmodified JSON payload.
    pub fn new(event: Option<String>, data: Value) -> Self {
        let kind = AnthropicMessageStreamEventType::from_name(event.as_deref());
        Self { event, kind, data }
    }
}
