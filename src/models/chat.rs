//! OpenAI-compatible Chat Completions models.

use super::{ReasoningConfig, ReasoningEffort, ReasoningRequest};
use crate::models::tools::{ResponseFormat, Tool, ToolChoice};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

/// A `stop` value accepted by compatible Chat Completions endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(untagged)]
pub enum StopSequences {
    /// One stop sequence.
    One(String),
    /// Several stop sequences.
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

impl From<Vec<&str>> for StopSequences {
    fn from(value: Vec<&str>) -> Self {
        Self::Many(value.into_iter().map(ToOwned::to_owned).collect())
    }
}

impl StopSequences {
    /// Returns the configured sequences for validation or inspection.
    pub fn as_slice(&self) -> Vec<&str> {
        match self {
            Self::One(value) => vec![value],
            Self::Many(values) => values.iter().map(String::as_str).collect(),
        }
    }
}

/// Provider-routing options that may be sent as a Chat extension.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ProviderOptions {
    /// Preferred provider order, if the compatible service supports routing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub order: Option<Vec<String>>,
    /// Whether fallback routing is allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow_fallbacks: Option<bool>,
    /// Whether all request parameters must be supported by the selected route.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_parameters: Option<bool>,
    /// Provider sorting mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort: Option<String>,
    /// Explicit future provider fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl ProviderOptions {
    /// Creates routing options preferring one opaque provider label.
    pub fn preferred(provider: impl Into<String>) -> Self {
        Self {
            order: Some(vec![provider.into()]),
            ..Self::default()
        }
    }

    /// Sets provider preference order.
    pub fn with_order<I, S>(mut self, providers: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.order = Some(providers.into_iter().map(Into::into).collect());
        self
    }

    /// Sets fallback routing.
    pub fn with_allow_fallbacks(mut self, allow: bool) -> Self {
        self.allow_fallbacks = Some(allow);
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

impl From<ProviderOptions> for Value {
    fn from(value: ProviderOptions) -> Self {
        serde_json::to_value(value).unwrap_or(Value::Null)
    }
}

/// Streaming options for a Chat request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChatStreamOptions {
    /// Request a final usage chunk when supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_usage: Option<bool>,
    /// Future stream fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl ChatStreamOptions {
    /// Creates options requesting a final usage chunk.
    pub fn include_usage() -> Self {
        Self {
            include_usage: Some(true),
            ..Self::default()
        }
    }
}

impl From<Value> for ChatStreamOptions {
    fn from(value: Value) -> Self {
        serde_json::from_value(value).unwrap_or_default()
    }
}

/// Output modality for Chat requests.
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

/// Image output options for compatible Chat endpoints.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ChatImageConfig {
    /// Requested image size.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub size: Option<String>,
    /// Requested image quality.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quality: Option<String>,
    /// Future-compatible image fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// A simple text chat message.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ChatMessage {
    /// Message role.
    pub role: MessageRole,
    /// Plain text content.
    pub content: String,
}

impl ChatMessage {
    /// Creates a system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }

    /// Creates a user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    /// Creates an assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

/// Roles accepted by the compact Chat message helper.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// System instructions.
    System,
    /// User input.
    User,
    /// Assistant output.
    Assistant,
}

/// Roles accepted by the complete OpenAI-compatible message shape.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OpenAIMessageRole {
    /// System instructions.
    System,
    /// Developer instructions.
    Developer,
    /// User input.
    User,
    /// Assistant output.
    Assistant,
    /// Tool result.
    Tool,
}

/// OpenAI-compatible message content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OpenAIMessageContent {
    /// Plain text content.
    Text(String),
    /// Multimodal content parts.
    Parts(Vec<OpenAIContentPart>),
}

impl From<String> for OpenAIMessageContent {
    fn from(value: String) -> Self {
        Self::Text(value)
    }
}

impl From<&str> for OpenAIMessageContent {
    fn from(value: &str) -> Self {
        Self::Text(value.to_string())
    }
}

/// Stable OpenAI-compatible multimodal content part.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIContentPart {
    /// Text content.
    Text {
        /// Text value.
        text: String,
    },
    /// Image URL or data URL.
    ImageUrl {
        /// Image URL payload.
        image_url: OpenAIImageUrl,
    },
    /// Input audio payload.
    InputAudio {
        /// Audio data payload.
        input_audio: OpenAIInputAudio,
    },
    /// File input payload.
    File {
        /// File payload.
        file: OpenAIFile,
    },
}

/// Image URL content payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAIImageUrl {
    /// URL or data URL.
    pub url: String,
    /// Optional detail hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// Input audio content payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAIInputAudio {
    /// Base64 audio data.
    pub data: String,
    /// Audio format, such as `wav` or `mp3`.
    pub format: String,
}

/// File content payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAIFile {
    /// File identifier, URL, or data URL depending on the endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<String>,
    /// File URL when supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_url: Option<String>,
    /// Base64 file data when supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_data: Option<String>,
    /// MIME filename when supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filename: Option<String>,
}

impl OpenAIMessageContent {
    /// Creates text content.
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text(content.into())
    }

    /// Creates multimodal content.
    pub fn parts(parts: Vec<OpenAIContentPart>) -> Self {
        Self::Parts(parts)
    }
}

impl OpenAIContentPart {
    /// Creates a text part.
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text {
            text: content.into(),
        }
    }

    /// Creates an image URL part.
    pub fn image_url(url: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: OpenAIImageUrl {
                url: url.into(),
                detail: None,
            },
        }
    }

    /// Creates an image URL part with a detail hint.
    pub fn image_url_with_detail(url: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: OpenAIImageUrl {
                url: url.into(),
                detail: Some(detail.into()),
            },
        }
    }

    /// Creates an input-audio part.
    pub fn input_audio(data: impl Into<String>, format: impl Into<String>) -> Self {
        Self::InputAudio {
            input_audio: OpenAIInputAudio {
                data: data.into(),
                format: format.into(),
            },
        }
    }
}

/// OpenAI-compatible function call details.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct OpenAIFunctionCall {
    /// Function name.
    pub name: String,
    /// JSON-encoded arguments.
    pub arguments: String,
}

/// OpenAI-compatible assistant tool call.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenAIToolCall {
    /// Tool call identifier.
    pub id: String,
    /// Tool call type.
    pub r#type: String,
    /// Function call details.
    pub function: OpenAIFunctionCall,
    /// Provider-specific tool metadata preserved for valid replay payloads.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_content: Option<Value>,
}

/// Full OpenAI-compatible message used for tool and multimodal replay.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenAIChatMessage {
    /// Message role.
    pub role: OpenAIMessageRole,
    /// Text or multimodal content. Assistant tool-call messages may omit it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<OpenAIMessageContent>,
    /// Optional display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Assistant tool calls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
    /// Tool result identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Refusal text when supplied by the endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    /// Opaque reasoning content that some compatible endpoints require for replay.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_content: Option<String>,
    /// Opaque structured reasoning details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_details: Option<Value>,
    /// Future-compatible message fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

impl OpenAIChatMessage {
    /// Creates a system message.
    pub fn system(content: impl Into<OpenAIMessageContent>) -> Self {
        Self::with_content(OpenAIMessageRole::System, Some(content.into()))
    }

    /// Creates a developer message.
    pub fn developer(content: impl Into<OpenAIMessageContent>) -> Self {
        Self::with_content(OpenAIMessageRole::Developer, Some(content.into()))
    }

    /// Creates a user message.
    pub fn user(content: impl Into<OpenAIMessageContent>) -> Self {
        Self::with_content(OpenAIMessageRole::User, Some(content.into()))
    }

    /// Creates an assistant message.
    pub fn assistant(content: impl Into<OpenAIMessageContent>) -> Self {
        Self::with_content(OpenAIMessageRole::Assistant, Some(content.into()))
    }

    /// Creates an assistant message containing only tool calls.
    pub fn assistant_with_tool_calls(tool_calls: Vec<OpenAIToolCall>) -> Self {
        Self {
            role: OpenAIMessageRole::Assistant,
            content: None,
            name: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
            refusal: None,
            reasoning_content: None,
            reasoning_details: None,
            extra: BTreeMap::new(),
        }
    }

    /// Creates a tool result message.
    pub fn tool(tool_call_id: impl Into<String>, content: impl Into<OpenAIMessageContent>) -> Self {
        Self {
            role: OpenAIMessageRole::Tool,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: Some(tool_call_id.into()),
            refusal: None,
            reasoning_content: None,
            reasoning_details: None,
            extra: BTreeMap::new(),
        }
    }

    /// Creates a message with explicit role/content.
    pub fn with_content(role: OpenAIMessageRole, content: Option<OpenAIMessageContent>) -> Self {
        Self {
            role,
            content,
            name: None,
            tool_calls: None,
            tool_call_id: None,
            refusal: None,
            reasoning_content: None,
            reasoning_details: None,
            extra: BTreeMap::new(),
        }
    }

    /// Sets the optional display name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }
}

impl From<ChatMessage> for OpenAIChatMessage {
    fn from(value: ChatMessage) -> Self {
        let role = match value.role {
            MessageRole::System => OpenAIMessageRole::System,
            MessageRole::User => OpenAIMessageRole::User,
            MessageRole::Assistant => OpenAIMessageRole::Assistant,
        };
        Self::with_content(role, Some(OpenAIMessageContent::Text(value.content)))
    }
}

/// Shared OpenAI-compatible Chat Completions request.
///
/// The message type is generic so compact text requests and full tool/multimodal
/// replay share one request implementation instead of duplicating every field.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatRequest<M> {
    /// Model identifier.
    pub model: String,
    /// Conversation history.
    pub messages: Vec<M>,
    /// Sampling temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    /// Legacy maximum completion token field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    /// Modern maximum completion token field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_completion_tokens: Option<u32>,
    /// Nucleus sampling value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    /// Frequency penalty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    /// Presence penalty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    /// Stop string or strings.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<StopSequences>,
    /// End-user identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    /// Optional opaque provider-routing value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<Value>,
    /// Whether to stream the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
    /// Stream options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<ChatStreamOptions>,
    /// Token logit bias object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<Value>,
    /// Whether to include log probabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,
    /// Number of top log probabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,
    /// Number of choices.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,
    /// Structured output format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,
    /// Function tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,
    /// Tool selection policy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,
    /// Whether parallel tool calls are allowed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parallel_tool_calls: Option<bool>,
    /// Deterministic seed when supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub seed: Option<i64>,
    /// Prompt-cache key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,
    /// Provider extension options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_options: Option<Value>,
    /// Prompt-cache retention hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_retention: Option<String>,
    /// Structured or boolean reasoning control.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningRequest>,
    /// First-class effort label.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<ReasoningEffort>,
    /// Include reasoning where supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_reasoning: Option<bool>,
    /// String metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<HashMap<String, String>>,
    /// Service-tier hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    /// Persist the response when supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store: Option<bool>,
    /// Stable safety identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub safety_identifier: Option<String>,
    /// Requested output modalities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modalities: Option<Vec<ChatModality>>,
    /// Audio generation options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub audio: Option<Value>,
    /// Prediction optimization payload.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prediction: Option<Value>,
    /// Output verbosity hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub verbosity: Option<String>,
    /// Hosted web-search options.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub web_search_options: Option<Value>,
    /// Legacy function definitions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub functions: Option<Vec<Value>>,
    /// Legacy function-call directive.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<Value>,
    /// Future-compatible top-level fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Compact text Chat request.
pub type ChatCompletionRequest = ChatRequest<ChatMessage>;
/// Full OpenAI-compatible Chat request.
pub type OpenAIChatCompletionRequest = ChatRequest<OpenAIChatMessage>;

impl<M> ChatRequest<M> {
    /// Creates a request with a model and message history.
    pub fn new(model: impl Into<String>, messages: Vec<M>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            max_tokens: None,
            max_completion_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            user: None,
            provider: None,
            stream: None,
            stream_options: None,
            logit_bias: None,
            logprobs: None,
            top_logprobs: None,
            n: None,
            response_format: None,
            tools: None,
            tool_choice: None,
            parallel_tool_calls: None,
            seed: None,
            prompt_cache_key: None,
            provider_options: None,
            prompt_cache_retention: None,
            reasoning: None,
            reasoning_effort: None,
            include_reasoning: None,
            metadata: None,
            service_tier: None,
            store: None,
            safety_identifier: None,
            modalities: None,
            audio: None,
            prediction: None,
            verbosity: None,
            web_search_options: None,
            functions: None,
            function_call: None,
            extra: BTreeMap::new(),
        }
    }

    /// Sets temperature, clamped to the documented inclusive range.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature.clamp(0.0, 2.0));
        self
    }

    /// Sets the legacy maximum token field.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the modern maximum completion token field.
    pub fn with_max_completion_tokens(mut self, max_tokens: u32) -> Self {
        self.max_completion_tokens = Some(max_tokens);
        self
    }

    /// Sets top-p, clamped to the documented range.
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p.clamp(0.0, 1.0));
        self
    }

    /// Sets frequency penalty, clamped to the documented range.
    pub fn with_frequency_penalty(mut self, value: f32) -> Self {
        self.frequency_penalty = Some(value.clamp(-2.0, 2.0));
        self
    }

    /// Sets presence penalty, clamped to the documented range.
    pub fn with_presence_penalty(mut self, value: f32) -> Self {
        self.presence_penalty = Some(value.clamp(-2.0, 2.0));
        self
    }

    /// Sets the number of choices to generate.
    pub fn with_n(mut self, n: u32) -> Self {
        self.n = Some(n);
        self
    }

    /// Sets one or more stop sequences.
    pub fn with_stop(mut self, stop: impl Into<StopSequences>) -> Self {
        self.stop = Some(stop.into());
        self
    }

    /// Sets the end-user identifier.
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Sets a token logit-bias object.
    pub fn with_logit_bias(mut self, value: Value) -> Self {
        self.logit_bias = Some(value);
        self
    }

    /// Requests token log probabilities.
    pub fn with_logprobs(mut self, value: bool) -> Self {
        self.logprobs = Some(value);
        self
    }

    /// Sets the number of top log-probability alternatives.
    pub fn with_top_logprobs(mut self, value: u32) -> Self {
        self.top_logprobs = Some(value);
        self
    }

    /// Sets an opaque provider-routing string.
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(Value::String(provider.into()));
        self
    }

    /// Sets an opaque provider-routing object.
    pub fn with_provider_value(mut self, provider: Value) -> Self {
        self.provider = Some(provider);
        self
    }

    /// Sets whether the request streams.
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Sets stream options.
    pub fn with_stream_options(mut self, options: impl Into<ChatStreamOptions>) -> Self {
        self.stream_options = Some(options.into());
        self
    }

    /// Sets response format.
    pub fn with_response_format(mut self, format: ResponseFormat) -> Self {
        self.response_format = Some(format);
        self
    }

    /// Sets function tools.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Sets tool-choice policy.
    pub fn with_tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    /// Sets whether multiple tool calls may be emitted in one turn.
    pub fn with_parallel_tool_calls(mut self, value: bool) -> Self {
        self.parallel_tool_calls = Some(value);
        self
    }

    /// Sets a deterministic seed when supported by the endpoint.
    pub fn with_seed(mut self, value: i64) -> Self {
        self.seed = Some(value);
        self
    }

    /// Sets a prompt-cache key.
    pub fn with_prompt_cache_key(mut self, value: impl Into<String>) -> Self {
        self.prompt_cache_key = Some(value.into());
        self
    }

    /// Sets a prompt-cache retention hint.
    pub fn with_prompt_cache_retention(mut self, value: impl Into<String>) -> Self {
        self.prompt_cache_retention = Some(value.into());
        self
    }

    /// Sets reasoning as a boolean, structured config, or explicit raw value.
    pub fn with_reasoning(mut self, reasoning: impl Into<ReasoningRequest>) -> Self {
        self.reasoning = Some(reasoning.into());
        self
    }

    /// Sets an exact first-class effort label.
    pub fn with_reasoning_effort(mut self, effort: impl Into<ReasoningEffort>) -> Self {
        self.reasoning_effort = Some(effort.into());
        self
    }

    /// Sets an explicit numeric reasoning budget inside `reasoning.max_tokens`.
    pub fn with_reasoning_budget(mut self, max_tokens: u32) -> Self {
        self.reasoning = Some(ReasoningConfig::manual_budget(max_tokens).into());
        self
    }

    /// Sets a structured reasoning object.
    pub fn with_reasoning_config(mut self, config: super::ReasoningConfig) -> Self {
        self.reasoning = Some(config.into());
        self
    }

    /// Requests reasoning content where the endpoint supports it.
    pub fn with_include_reasoning(mut self, include: bool) -> Self {
        self.include_reasoning = Some(include);
        self
    }

    /// Sets metadata.
    pub fn with_metadata(mut self, metadata: HashMap<String, String>) -> Self {
        self.metadata = Some(metadata);
        self
    }

    /// Sets service tier.
    pub fn with_service_tier(mut self, service_tier: impl Into<String>) -> Self {
        self.service_tier = Some(service_tier.into());
        self
    }

    /// Sets provider extension options.
    pub fn with_provider_options(mut self, options: impl Into<Value>) -> Self {
        self.provider_options = Some(options.into());
        self
    }

    /// Sets the output modalities.
    pub fn with_modalities(mut self, modalities: Vec<ChatModality>) -> Self {
        self.modalities = Some(modalities);
        self
    }

    /// Sets audio output options.
    pub fn with_audio(mut self, audio: Value) -> Self {
        self.audio = Some(audio);
        self
    }

    /// Sets a prediction optimization payload.
    pub fn with_prediction(mut self, prediction: Value) -> Self {
        self.prediction = Some(prediction);
        self
    }

    /// Sets an output verbosity hint.
    pub fn with_verbosity(mut self, verbosity: impl Into<String>) -> Self {
        self.verbosity = Some(verbosity.into());
        self
    }

    /// Sets hosted web-search options.
    pub fn with_web_search_options(mut self, options: Value) -> Self {
        self.web_search_options = Some(options);
        self
    }

    /// Sets the response-storage flag.
    pub fn with_store(mut self, store: bool) -> Self {
        self.store = Some(store);
        self
    }

    /// Sets a stable safety identifier.
    pub fn with_safety_identifier(mut self, identifier: impl Into<String>) -> Self {
        self.safety_identifier = Some(identifier.into());
        self
    }

    /// Sets legacy function definitions.
    pub fn with_functions(mut self, functions: Vec<Value>) -> Self {
        self.functions = Some(functions);
        self
    }

    /// Sets a legacy function-call directive.
    pub fn with_function_call(mut self, function_call: impl Into<Value>) -> Self {
        self.function_call = Some(function_call.into());
        self
    }

    /// Adds an explicit future-compatible field.
    pub fn with_extra(mut self, key: impl Into<String>, value: Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }

    /// Validates protocol-level parameter constraints without inspecting the model name.
    pub fn validate_openai_compatibility(&self) -> Result<(), String> {
        if let Some(value) = self.temperature
            && (!value.is_finite() || !(0.0..=2.0).contains(&value))
        {
            return Err(format!(
                "temperature must be between 0.0 and 2.0, got {value}"
            ));
        }
        if let Some(value) = self.top_p
            && (!value.is_finite() || !(0.0..=1.0).contains(&value))
        {
            return Err(format!("top_p must be between 0.0 and 1.0, got {value}"));
        }
        for (name, value) in [
            ("frequency_penalty", self.frequency_penalty),
            ("presence_penalty", self.presence_penalty),
        ] {
            if let Some(value) = value
                && (!value.is_finite() || !(-2.0..=2.0).contains(&value))
            {
                return Err(format!("{name} must be between -2.0 and 2.0, got {value}"));
            }
        }
        if self.max_tokens == Some(0) || self.max_completion_tokens == Some(0) {
            return Err("maximum token counts must be greater than zero".to_string());
        }
        if self.top_logprobs.is_some_and(|value| value > 20) {
            return Err("top_logprobs must be between 0 and 20".to_string());
        }
        if self.n == Some(0) {
            return Err("n must be greater than zero".to_string());
        }
        if let Some(stop) = &self.stop {
            let values = stop.as_slice();
            if values.len() > 4 {
                return Err("at most four stop sequences are supported".to_string());
            }
            if values
                .iter()
                .any(|value| value.is_empty() || value.len() > 64)
            {
                return Err("stop sequences must be 1 to 64 bytes long".to_string());
            }
        }
        if let Some(ReasoningRequest::Config(config)) = &self.reasoning {
            config.validate()?;
        }
        Ok(())
    }
}

/// Chat completion response shared by compact and full message variants.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatResponse<M = ChatMessage> {
    /// Completion identifier.
    pub id: String,
    /// Object label.
    pub object: String,
    /// Creation timestamp.
    pub created: u64,
    /// Model identifier.
    pub model: String,
    /// Returned choices.
    pub choices: Vec<ChatChoice<M>>,
    /// Token usage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<super::Usage>,
    /// Service tier, when returned.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_tier: Option<String>,
    /// Future-compatible response fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Compact text chat completion response.
pub type ChatCompletionResponse = ChatResponse<ChatMessage>;
/// Full OpenAI-compatible chat completion response.
pub type OpenAIChatCompletionResponse = ChatResponse<OpenAIChatMessage>;

/// One returned Chat choice.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatChoice<M = ChatMessage> {
    /// Choice index.
    pub index: u32,
    /// Generated message.
    pub message: M,
    /// Stop reason, when supplied.
    pub finish_reason: Option<String>,
    /// Future-compatible choice fields.
    #[serde(flatten, default, skip_serializing_if = "BTreeMap::is_empty")]
    pub extra: BTreeMap<String, Value>,
}

/// Full OpenAI-compatible choice alias.
pub type OpenAIChatChoice = ChatChoice<OpenAIChatMessage>;
