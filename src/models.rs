use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::HashMap;

fn map_is_empty(value: &HashMap<String, serde_json::Value>) -> bool {
    value.is_empty()
}

/// Represents a single message in a chat conversation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    /// The role of the message author.
    pub role: MessageRole,
    /// The content of the message.
    pub content: String,
}

/// The role of a message's author.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    /// A message from the system, setting the context or instructions for the assistant.
    System,
    /// A message from the user.
    User,
    /// A message from the assistant.
    Assistant,
}

/// The role of an OpenAI-compatible chat message author.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum OpenAIMessageRole {
    /// A message from the system.
    System,
    /// A message from the user.
    User,
    /// A message from the assistant.
    Assistant,
    /// A tool result message.
    Tool,
}

/// OpenAI-compatible chat message content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum OpenAIMessageContent {
    /// Plain text content.
    Text(String),
    /// Multimodal content parts.
    Parts(Vec<OpenAIContentPart>),
}

/// OpenAI-compatible multimodal content part.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenAIContentPart {
    /// Text content part.
    Text {
        /// The text content.
        text: String,
    },
    /// Image URL content part.
    ImageUrl {
        /// Image URL payload.
        image_url: OpenAIImageUrl,
    },
}

/// OpenAI-compatible image URL payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenAIImageUrl {
    /// Image URL or data URI.
    pub url: String,
    /// Optional detail level hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

/// OpenAI-compatible function call payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenAIFunctionCall {
    /// Function name.
    pub name: String,
    /// JSON-encoded function arguments.
    pub arguments: String,
}

/// OpenAI-compatible tool call payload.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenAIToolCall {
    /// Tool call ID.
    pub id: String,
    /// Tool type (typically `function`).
    pub r#type: String,
    /// Optional provider-specific metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extra_content: Option<serde_json::Value>,
    /// Function call details.
    pub function: OpenAIFunctionCall,
}

/// OpenAI-compatible chat message with full tool-call replay support.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct OpenAIChatMessage {
    /// The role of the message author.
    pub role: OpenAIMessageRole,
    /// Message content. Assistant tool-call messages may omit content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<OpenAIMessageContent>,
    /// Optional display name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Assistant tool calls attached to this message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<OpenAIToolCall>>,
    /// Tool call ID associated with a `tool` role message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

/// The search provider to use for web research.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResearchProvider {
    /// Use Exa (formerly Metaphor) for high-quality semantic search.
    #[default]
    Exa,
    /// Use Tavily for comprehensive web search and content extraction.
    Tavily,
    /// Automatically select the best provider based on the query.
    Auto,
}

/// The depth of the research operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum ResearchDepth {
    /// Basic search (faster, lower cost).
    #[default]
    Basic,
    /// Deep search (more thorough, higher cost, includes more context).
    Advanced,
}

/// Represents a request to create a chat completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// The identifier of the model to use for the completion (e.g., "gpt-4o", "claude-sonnet-4").
    pub model: String,

    /// A list of messages that form the conversation history.
    pub messages: Vec<ChatMessage>,

    /// The sampling temperature to use, between 0.0 and 2.0. Higher values will make the output
    /// more random, while lower values will make it more focused and deterministic.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// The maximum number of tokens to generate in the completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// The nucleus sampling parameter. The model considers the results of the tokens with `top_p`
    /// probability mass. So, 0.1 means only the tokens comprising the top 10% probability mass are considered.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// A penalty applied to new tokens based on their frequency in the text so far.
    /// It decreases the model's likelihood to repeat the same line verbatim.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// A penalty applied to new tokens based on whether they appear in the text so far.
    /// It increases the model's likelihood to talk about new topics.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// A list of sequences that will cause the model to stop generating further tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// A unique identifier representing your end-user, which can help in monitoring and
    /// tracking conversations.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// A hint to the router about which provider to use for the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// If set to `true`, the response will be streamed as a series of events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Modify the likelihood of specified tokens appearing in the completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<serde_json::Value>,

    /// Whether to return log probabilities of the output tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// An integer between 0 and 20 specifying the number of most likely tokens to return at each token position.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,

    /// How many chat completion choices to generate for each input message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,

    /// An object specifying the format that the model must output.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// A list of tools the model may call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Controls which (if any) tool is called by the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Configuration for thinking capabilities (Gemini 3 and 2.5 series).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_config: Option<ThinkingConfig>,
}

/// OpenAI-compatible request payload with full message replay support.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChatCompletionRequest {
    /// The identifier of the model to use for the completion.
    pub model: String,

    /// Full OpenAI-compatible message history.
    pub messages: Vec<OpenAIChatMessage>,

    /// The sampling temperature to use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// The maximum number of tokens to generate in the completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,

    /// Nucleus sampling parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Frequency penalty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,

    /// Presence penalty.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,

    /// Stop sequences.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,

    /// End-user identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Router/provider hint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,

    /// If true, the response will be streamed as SSE events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Logit bias map.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logit_bias: Option<serde_json::Value>,

    /// Whether to return log probabilities.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logprobs: Option<bool>,

    /// Number of top log probabilities to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_logprobs: Option<u32>,

    /// Number of completion choices to generate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub n: Option<u32>,

    /// Structured response format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<ResponseFormat>,

    /// Tools available to the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<Tool>>,

    /// Tool selection strategy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<ToolChoice>,

    /// Gemini thinking configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_config: Option<ThinkingConfig>,

    /// Anthropic extended-thinking configuration (`thinking.budget_tokens`).
    /// Serialised as the `thinking` top-level field so it is passed through to
    /// OpenRouter/Anthropic as `{"type":"enabled","budget_tokens":N}`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking: Option<serde_json::Value>,
}

/// Represents the response from a chat completion request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// A unique identifier for the chat completion.
    pub id: String,

    /// The type of object, which is always "chat.completion".
    pub object: String,

    /// The Unix timestamp (in seconds) of when the completion was created.
    pub created: u64,

    /// The model that was used for the completion.
    pub model: String,

    /// A list of chat completion choices.
    pub choices: Vec<ChatChoice>,

    /// Information about the token usage for this completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// OpenAI-compatible chat completion response with tool-call aware messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChatCompletionResponse {
    /// A unique identifier for the chat completion.
    pub id: String,

    /// The type of object, which is always `chat.completion`.
    pub object: String,

    /// The Unix timestamp (in seconds) of when the completion was created.
    pub created: u64,

    /// The model that was used for the completion.
    pub model: String,

    /// A list of chat completion choices.
    pub choices: Vec<OpenAIChatChoice>,

    /// Token usage information for this completion.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Represents a chunk of a streaming chat completion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    /// A unique identifier for the chat completion.
    pub id: String,

    /// The type of object, which is always "chat.completion.chunk".
    pub object: String,

    /// The Unix timestamp (in seconds) of when the completion was created.
    pub created: u64,

    /// The model that was used for the completion.
    pub model: String,

    /// A list of chat completion choices.
    pub choices: Vec<ChatCompletionChunkChoice>,
}

/// Represents a single choice in a streaming chat completion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunkChoice {
    /// The index of the choice in the list of choices.
    pub index: u32,

    /// A delta payload with the content that has changed since the last chunk.
    pub delta: ChatCompletionChunkDelta,

    /// The reason the model stopped generating tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Represents the delta payload of a streaming chat completion chunk.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionChunkDelta {
    /// The role of the message author.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<MessageRole>,

    /// The content of the message.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// The thinking content (for Gemini 3 models).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<String>,
}

/// Represents a single choice in a chat completion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    /// The index of the choice in the list of choices.
    pub index: u32,

    /// The message generated by the model.
    pub message: ChatMessage,

    /// The reason the model stopped generating tokens.
    pub finish_reason: String,
}

/// Represents a single choice in an OpenAI-compatible chat completion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenAIChatChoice {
    /// The index of the choice in the list of choices.
    pub index: u32,

    /// The tool-call aware message generated by the model.
    pub message: OpenAIChatMessage,

    /// The reason the model stopped generating tokens.
    pub finish_reason: String,
}

/// Represents the token usage statistics for a chat completion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// The number of tokens in the prompt.
    pub prompt_tokens: u32,

    /// The number of tokens in the generated completion.
    pub completion_tokens: u32,

    /// The total number of tokens used in the request (prompt + completion).
    pub total_tokens: u32,
}

/// Represents the health status of the Rainy API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// The overall status of the API (e.g., "healthy", "degraded").
    pub status: String,

    /// The timestamp of when the health check was performed.
    pub timestamp: String,

    /// The uptime of the system in seconds.
    pub uptime: f64,

    /// The status of individual services.
    pub services: ServiceStatus,
}

/// Represents the status of individual backend services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// The status of the database connection.
    pub database: bool,

    /// The status of the Redis connection, if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis: Option<bool>,

    /// The overall status of the connections to AI providers.
    pub providers: bool,
}

/// Represents the available models and providers.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AvailableModels {
    /// A map where keys are provider names and values are lists of model names.
    #[serde(default)]
    pub providers: HashMap<String, Vec<String>>,

    /// The total number of available models across all providers.
    #[serde(default)]
    pub total_models: usize,

    /// A list of provider names that are currently active and available.
    #[serde(default)]
    pub active_providers: Vec<String>,
}

/// Represents information about credit usage for a request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditInfo {
    /// The number of credits available before the request.
    pub current_credits: f64,

    /// The estimated number of credits that the request will cost.
    pub estimated_cost: f64,

    /// The estimated number of credits remaining after the request.
    pub credits_after_request: f64,

    /// The date when the credit balance is next scheduled to be reset.
    pub reset_date: String,
}

/// Represents metadata extracted from the response headers of an API request.
#[derive(Debug, Clone)]
pub struct RequestMetadata {
    /// The time taken for the request to complete, in milliseconds.
    pub response_time: Option<u64>,

    /// The AI provider that handled the request.
    pub provider: Option<String>,

    /// The number of tokens used in the request.
    pub tokens_used: Option<u32>,

    /// The number of credits used for the request.
    pub credits_used: Option<f64>,

    /// The number of credits remaining after the request.
    pub credits_remaining: Option<f64>,

    /// The unique ID of the request, for tracking and debugging.
    pub request_id: Option<String>,

    /// Count of non-blocking compatibility warnings returned by Rainy.
    pub compat_warnings: Option<u32>,

    /// Response mode selected by Rainy (`raw` or `envelope`).
    pub response_mode: Option<String>,

    /// Billing plan used for this request.
    pub billing_plan: Option<String>,

    /// Credits charged for the request.
    pub rainy_credits_charged: Option<f64>,

    /// Markup percent applied by gateway pricing.
    pub rainy_markup_percent: Option<f64>,

    /// Remaining daily credits reported by Rainy.
    pub rainy_daily_credits_remaining: Option<String>,
}

/// OpenRouter/Rainy Responses API request payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsesRequest {
    /// The identifier of the model to use.
    pub model: String,

    /// Input payload accepted by the Responses API (string, object, or array).
    pub input: serde_json::Value,

    /// If true, the response will be streamed as SSE events.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,

    /// Responses tool definitions and/or custom tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<Vec<serde_json::Value>>,

    /// Tool selection strategy.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<serde_json::Value>,

    /// Structured output format.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<serde_json::Value>,

    /// Sampling temperature.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,

    /// Top-p nucleus sampling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,

    /// Maximum number of output tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_output_tokens: Option<u32>,

    /// End-user identifier (legacy fallback accepted by Rainy).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,

    /// Prompt cache key for routing/cache optimization.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt_cache_key: Option<String>,

    /// Reasoning configuration object (provider/model dependent).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<serde_json::Value>,

    /// Forward-compatible extra parameters.
    #[serde(flatten, skip_serializing_if = "map_is_empty", default)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl ResponsesRequest {
    /// Creates a new Responses request from an arbitrary input payload.
    pub fn new(model: impl Into<String>, input: serde_json::Value) -> Self {
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
            user: None,
            prompt_cache_key: None,
            reasoning: None,
            extra: HashMap::new(),
        }
    }

    /// Convenience constructor for plain text input.
    pub fn text(model: impl Into<String>, input_text: impl Into<String>) -> Self {
        Self::new(model, serde_json::Value::String(input_text.into()))
    }

    /// Sets streaming mode.
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Sets reasoning configuration object.
    pub fn with_reasoning(mut self, reasoning: serde_json::Value) -> Self {
        self.reasoning = Some(reasoning);
        self
    }

    /// Convenience helper to set reasoning effort (`low`, `medium`, `high`).
    pub fn with_reasoning_effort(mut self, effort: impl Into<String>) -> Self {
        self.reasoning = Some(serde_json::json!({ "effort": effort.into() }));
        self
    }

    /// Sets max output tokens.
    pub fn with_max_output_tokens(mut self, max_output_tokens: u32) -> Self {
        self.max_output_tokens = Some(max_output_tokens);
        self
    }

    /// Sets prompt cache key.
    pub fn with_prompt_cache_key(mut self, prompt_cache_key: impl Into<String>) -> Self {
        self.prompt_cache_key = Some(prompt_cache_key.into());
        self
    }

    /// Sets user identifier.
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Sets tool definitions array directly.
    pub fn with_tools(mut self, tools: Vec<serde_json::Value>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Adds a function tool using Responses-style shape.
    pub fn add_function_tool(
        mut self,
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: serde_json::Value,
    ) -> Self {
        let mut tools = self.tools.unwrap_or_default();
        tools.push(serde_json::json!({
            "type": "function",
            "name": name.into(),
            "description": description.into(),
            "parameters": parameters
        }));
        self.tools = Some(tools);
        self
    }

    /// Adds a custom extra parameter for forward compatibility.
    pub fn with_extra(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.extra.insert(key.into(), value);
        self
    }
}

/// Responses API usage object (partial, forward-compatible).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponsesUsage {
    /// Number of input tokens consumed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_tokens: Option<u32>,
    /// Number of output tokens generated.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens: Option<u32>,
    /// Number of tokens used for cache creation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_creation_input_tokens: Option<u32>,
    /// Number of tokens read from cache.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cache_read_input_tokens: Option<u32>,
    /// Detailed breakdown of output tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_tokens_details: Option<serde_json::Value>,
    /// Detailed breakdown of completion tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion_tokens_details: Option<serde_json::Value>,
    /// Additional provider-specific usage fields.
    #[serde(flatten, default)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Responses API raw response payload.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponsesApiResponse {
    /// Unique identifier for the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// Object type identifier.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    /// Model used for the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    /// Plain text output content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_text: Option<String>,
    /// Structured output items.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<Vec<serde_json::Value>>,
    /// Token usage information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<ResponsesUsage>,
    /// Additional provider-specific response fields.
    #[serde(flatten, default)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Non-blocking compatibility warning emitted by Rainy in envelope mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompatWarning {
    /// Warning code identifier.
    pub code: String,
    /// Human-readable warning message.
    pub message: String,
    /// JSON path to the field that triggered the warning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

/// Features used by request (reported in envelope mode).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeaturesUsed {
    /// Whether reasoning/thinking was used.
    pub reasoning: bool,
    /// Whether image input was provided.
    #[serde(rename = "imageInput")]
    pub image_input: bool,
    /// Whether tool calling was used.
    pub tools: bool,
    /// Whether structured output was requested.
    #[serde(rename = "structuredOutput")]
    pub structured_output: bool,
}

/// Reasoning summary metadata reported by Rainy in envelope mode.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReasoningMeta {
    /// Whether reasoning was present in the response.
    pub present: bool,
    /// Whether a reasoning summary was provided.
    pub summary_present: bool,
    /// Number of tokens used for reasoning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<u32>,
}

/// Rainy envelope metadata (partial, forward-compatible).
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RainyEnvelopeMeta {
    /// Billing plan identifier.
    #[serde(
        rename = "billingPlan",
        alias = "billing_plan",
        skip_serializing_if = "Option::is_none"
    )]
    pub billing_plan: Option<String>,
    /// Credits charged for the request.
    #[serde(
        rename = "creditsCharged",
        alias = "credits_charged",
        skip_serializing_if = "Option::is_none"
    )]
    pub credits_charged: Option<f64>,
    /// Markup percentage applied to pricing.
    #[serde(
        rename = "markupPercent",
        alias = "markup_percent",
        skip_serializing_if = "Option::is_none"
    )]
    pub markup_percent: Option<f64>,
    /// Daily credits remaining for the user.
    #[serde(
        rename = "dailyCreditsRemaining",
        alias = "daily_credits_remaining",
        skip_serializing_if = "Option::is_none"
    )]
    pub daily_credits_remaining: Option<String>,
    /// Compatibility warnings emitted during processing.
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
    /// Reasoning metadata for the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<ReasoningMeta>,
    /// Additional envelope metadata fields.
    #[serde(flatten, default)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Standard Rainy success envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RainyEnvelope<T> {
    /// Whether the request was successful.
    pub success: bool,
    /// The response data payload.
    pub data: T,
    /// Optional envelope metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<RainyEnvelopeMeta>,
}

/// Response stream SSE event payload (dynamic by design).
pub type ResponsesStreamEvent = serde_json::Value;

/// Model architecture metadata returned by `/models/catalog`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelArchitecture {
    /// Supported input modalities (e.g., "text", "image").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub input_modalities: Option<Vec<String>>,
    /// Supported output modalities (e.g., "text").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_modalities: Option<Vec<String>>,
    /// Tokenizer used by the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenizer: Option<String>,
    /// Instruction type supported by the model.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instruct_type: Option<String>,
}

/// Capability flag can be boolean or `"unknown"`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum CapabilityFlag {
    /// Boolean capability flag.
    Bool(bool),
    /// Text-based capability flag (e.g., "unknown").
    Text(String),
}

/// Rainy capability hints returned by `/models/catalog`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RainyCapabilities {
    /// Whether the model supports reasoning/thinking.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning: Option<CapabilityFlag>,
    /// Whether the model supports image input.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_input: Option<CapabilityFlag>,
    /// Whether the model supports tool calling.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tools: Option<CapabilityFlag>,
    /// Whether the model supports structured output formats.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub response_format: Option<CapabilityFlag>,
}

/// Provider-specific reasoning profile.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ReasoningProvider {
    /// OpenAI provider.
    Openai,
    /// Google provider.
    Google,
    /// Anthropic provider.
    Anthropic,
    /// Other providers.
    Other,
}

/// Thinking budget range metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ThinkingBudget {
    /// Minimum budget value.
    pub min: i32,
    /// Maximum budget value.
    pub max: i32,
    /// Dynamic budget value if applicable.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dynamic_value: Option<i32>,
    /// Value that disables thinking budget.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_value: Option<i32>,
}

/// Reasoning controls available for a model.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ReasoningControls {
    /// Parameters observed for reasoning control.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_parameters: Option<Vec<String>>,
    /// Whether reasoning toggle is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_toggle: Option<bool>,
    /// Whether reasoning effort control is supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_effort: Option<bool>,
    /// Available effort levels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effort: Option<Vec<String>>,
    /// Available thinking levels.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<Vec<String>>,
    /// Thinking budget configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_budget: Option<ThinkingBudget>,
}

/// Provider profile for reasoning/thinking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReasoningProfile {
    /// The provider this profile applies to.
    pub provider: ReasoningProvider,
    /// JSON path to the reasoning parameter.
    pub parameter_path: String,
    /// Available values for the parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
    /// Additional notes about this profile.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Reasoning toggle paths for clients.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ReasoningToggle {
    /// Parameter path to enable reasoning.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_param: Option<String>,
    /// Parameter path to include reasoning in response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_reasoning_param: Option<String>,
}

/// Reasoning capability block in `rainy_capabilities_v2`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RainyReasoningCapabilitiesV2 {
    /// Whether reasoning is supported.
    pub supported: bool,
    /// Available reasoning controls.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub controls: Option<ReasoningControls>,
    /// Provider-specific reasoning profiles.
    #[serde(default)]
    pub profiles: Vec<ReasoningProfile>,
    /// Reasoning toggle configuration.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub toggle: Option<ReasoningToggle>,
}

/// Multimodal capability block in `rainy_capabilities_v2`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RainyMultimodalCapabilitiesV2 {
    /// Supported input modalities.
    #[serde(default)]
    pub input: Vec<String>,
    /// Supported output modalities.
    #[serde(default)]
    pub output: Vec<String>,
}

/// Parameter capability block in `rainy_capabilities_v2`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RainyParametersCapabilitiesV2 {
    /// Accepted parameter names.
    #[serde(default)]
    pub accepted: Vec<String>,
}

/// Full v2 capability block returned by `/models/catalog`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct RainyCapabilitiesV2 {
    /// Multimodal capabilities.
    pub multimodal: RainyMultimodalCapabilitiesV2,
    /// Reasoning capabilities.
    pub reasoning: RainyReasoningCapabilitiesV2,
    /// Parameter capabilities.
    pub parameters: RainyParametersCapabilitiesV2,
}

/// Pricing metadata for model ranking helpers.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelPricing {
    /// Prompt token pricing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prompt: Option<String>,
    /// Completion token pricing.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completion: Option<String>,
}

/// Model entry returned by `/models/catalog`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ModelCatalogItem {
    /// Unique model identifier.
    pub id: String,
    /// Human-readable model name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Maximum context length in tokens.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context_length: Option<u32>,
    /// Model pricing information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pricing: Option<ModelPricing>,
    /// Supported API parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub supported_parameters: Option<Vec<String>>,
    /// Model architecture metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub architecture: Option<ModelArchitecture>,
    /// Rainy capability hints (v1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rainy_capabilities: Option<RainyCapabilities>,
    /// Rainy capability hints (v2).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rainy_capabilities_v2: Option<RainyCapabilitiesV2>,
    /// Additional model metadata.
    #[serde(flatten, default)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Reasoning mode expected by the caller when selecting models.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ReasoningMode {
    /// Effort-based reasoning control.
    Effort,
    /// Thinking level-based reasoning control.
    ThinkingLevel,
    /// Thinking budget-based reasoning control.
    ThinkingBudget,
}

/// Selector criteria for model discovery from `/models/catalog`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ModelSelectionCriteria {
    /// Required input modalities.
    #[serde(default)]
    pub required_input_modalities: Vec<String>,
    /// Required output modalities.
    #[serde(default)]
    pub required_output_modalities: Vec<String>,
    /// Whether tool calling is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_tools: Option<bool>,
    /// Whether structured output is required.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub require_structured_output: Option<bool>,
    /// Required reasoning mode.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_mode: Option<ReasoningMode>,
    /// Reasoning value to match.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reasoning_value: Option<String>,
}

/// Builder preference for reasoning payload generation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReasoningPreference {
    /// Reasoning mode to use.
    pub mode: ReasoningMode,
    /// Reasoning value to apply.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    /// Thinking budget value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub budget: Option<i32>,
}

fn parse_price(value: Option<&str>) -> f64 {
    value
        .and_then(|raw| raw.parse::<f64>().ok())
        .filter(|v| v.is_finite())
        .unwrap_or(f64::MAX)
}

fn has_required_modalities(available: &[String], required: &[String]) -> bool {
    if required.is_empty() {
        return true;
    }

    required.iter().all(|modality| {
        available
            .iter()
            .any(|candidate| candidate.eq_ignore_ascii_case(modality))
    })
}

fn supports_reasoning_preference(
    capabilities: &RainyCapabilitiesV2,
    mode: &ReasoningMode,
    reasoning_value: Option<&str>,
) -> bool {
    if !capabilities.reasoning.supported {
        return false;
    }

    let controls = capabilities.reasoning.controls.as_ref();
    match mode {
        ReasoningMode::Effort => controls
            .map(|c| {
                c.reasoning_effort == Some(true) || c.effort.as_ref().is_some_and(|v| !v.is_empty())
            })
            .filter(|supported| *supported)
            .map(|_| {
                controls
                    .and_then(|c| c.effort.as_ref())
                    .map(|values| {
                        reasoning_value.is_none_or(|value| {
                            values
                                .iter()
                                .any(|candidate| candidate.eq_ignore_ascii_case(value))
                        })
                    })
                    .unwrap_or(reasoning_value.is_none())
            })
            .unwrap_or(false),
        ReasoningMode::ThinkingLevel => controls
            .and_then(|c| c.thinking_level.as_ref())
            .map(|values| {
                reasoning_value.is_none_or(|value| {
                    values
                        .iter()
                        .any(|candidate| candidate.eq_ignore_ascii_case(value))
                })
            })
            .unwrap_or(false),
        ReasoningMode::ThinkingBudget => {
            controls.and_then(|c| c.thinking_budget.as_ref()).is_some()
        }
    }
}

fn catalog_item_supports(item: &ModelCatalogItem, parameter: &str) -> bool {
    if let Some(v2) = &item.rainy_capabilities_v2 {
        return v2
            .parameters
            .accepted
            .iter()
            .any(|candidate| candidate == parameter);
    }

    item.supported_parameters
        .as_ref()
        .map(|params| params.iter().any(|candidate| candidate == parameter))
        .unwrap_or(false)
}

/// Select models from catalog and rank by prompt price, completion price, then context length desc.
pub fn select_models(
    models: &[ModelCatalogItem],
    criteria: &ModelSelectionCriteria,
) -> Vec<ModelCatalogItem> {
    let required_inputs: Vec<String> = criteria
        .required_input_modalities
        .iter()
        .map(|v| v.to_lowercase())
        .collect();
    let required_outputs: Vec<String> = criteria
        .required_output_modalities
        .iter()
        .map(|v| v.to_lowercase())
        .collect();

    let mut filtered: Vec<ModelCatalogItem> = models
        .iter()
        .filter(|item| {
            let Some(v2) = item.rainy_capabilities_v2.as_ref() else {
                return false;
            };
            let input: Vec<String> = v2
                .multimodal
                .input
                .iter()
                .map(|v| v.to_lowercase())
                .collect();
            let output: Vec<String> = v2
                .multimodal
                .output
                .iter()
                .map(|v| v.to_lowercase())
                .collect();

            if !has_required_modalities(&input, &required_inputs) {
                return false;
            }

            if !has_required_modalities(&output, &required_outputs) {
                return false;
            }

            if criteria.require_tools == Some(true) && !catalog_item_supports(item, "tools") {
                return false;
            }

            if criteria.require_structured_output == Some(true)
                && !catalog_item_supports(item, "response_format")
                && !catalog_item_supports(item, "structured_outputs")
            {
                return false;
            }

            if let Some(mode) = &criteria.reasoning_mode {
                let reasoning_value = criteria.reasoning_value.as_deref();
                if !supports_reasoning_preference(v2, mode, reasoning_value) {
                    return false;
                }
            }

            true
        })
        .cloned()
        .collect();

    filtered.sort_by(|a, b| {
        let a_prompt = parse_price(a.pricing.as_ref().and_then(|p| p.prompt.as_deref()));
        let b_prompt = parse_price(b.pricing.as_ref().and_then(|p| p.prompt.as_deref()));
        let prompt_cmp = a_prompt.partial_cmp(&b_prompt).unwrap_or(Ordering::Equal);
        if prompt_cmp != Ordering::Equal {
            return prompt_cmp;
        }

        let a_completion = parse_price(a.pricing.as_ref().and_then(|p| p.completion.as_deref()));
        let b_completion = parse_price(b.pricing.as_ref().and_then(|p| p.completion.as_deref()));
        let completion_cmp = a_completion
            .partial_cmp(&b_completion)
            .unwrap_or(Ordering::Equal);
        if completion_cmp != Ordering::Equal {
            return completion_cmp;
        }

        let a_context = a.context_length.unwrap_or_default();
        let b_context = b.context_length.unwrap_or_default();
        b_context.cmp(&a_context)
    });

    filtered
}

/// Build provider-aware reasoning payload from `rainy_capabilities_v2`.
pub fn build_reasoning_config(
    model: &ModelCatalogItem,
    preference: &ReasoningPreference,
) -> Option<serde_json::Value> {
    let v2 = model.rainy_capabilities_v2.as_ref()?;
    if !v2.reasoning.supported {
        return None;
    }

    let profiles = &v2.reasoning.profiles;
    let controls = v2.reasoning.controls.as_ref();
    match preference.mode {
        ReasoningMode::Effort => {
            let value = preference.value.clone()?;
            let supports_effort = controls
                .map(|c| {
                    c.reasoning_effort == Some(true)
                        || c.effort.as_ref().is_some_and(|v| !v.is_empty())
                })
                .unwrap_or(false);
            if !supports_effort {
                return None;
            }
            if let Some(efforts) = controls.and_then(|c| c.effort.as_ref()) {
                if !efforts.iter().any(|v| v.eq_ignore_ascii_case(&value)) {
                    return None;
                }
            }

            let effort_profile = profiles
                .iter()
                .find(|p| p.parameter_path == "reasoning.effort")?;
            match effort_profile.parameter_path.as_str() {
                "reasoning.effort" => Some(serde_json::json!({
                    "reasoning": { "effort": value }
                })),
                _ => None,
            }
        }
        ReasoningMode::ThinkingLevel => {
            let value = preference.value.clone()?;
            let supports = controls
                .and_then(|c| c.thinking_level.as_ref())
                .map(|levels| levels.iter().any(|v| v.eq_ignore_ascii_case(&value)))
                .unwrap_or(false);
            if !supports {
                return None;
            }
            let level_profile = profiles
                .iter()
                .find(|p| p.parameter_path == "thinking_config.thinking_level")?;
            if let Some(values) = &level_profile.values {
                if !values.iter().any(|v| v.eq_ignore_ascii_case(&value)) {
                    return None;
                }
            }
            Some(serde_json::json!({
                "thinking_config": { "thinking_level": value }
            }))
        }
        ReasoningMode::ThinkingBudget => {
            let budget = preference.budget?;
            let supports = controls.and_then(|c| c.thinking_budget.as_ref())?;
            if budget < supports.min || budget > supports.max {
                return None;
            }
            let budget_profile = profiles.iter().find(|p| {
                p.parameter_path == "thinking.budget_tokens"
                    || p.parameter_path == "thinking_config.thinking_budget"
            })?;

            if budget_profile.parameter_path == "thinking.budget_tokens" {
                return Some(serde_json::json!({
                    "thinking": { "budget_tokens": budget }
                }));
            }
            if budget_profile.parameter_path == "thinking_config.thinking_budget" {
                return Some(serde_json::json!({
                "thinking_config": { "thinking_budget": budget }
                }));
            }
            None
        }
    }
}

/// A collection of predefined model constants for convenience.
/// All models listed here are confirmed to be 100% OpenAI-compatible without parameter adaptations.
pub mod model_constants {
    // OpenAI models (fully compatible)
    /// Constant for the GPT-4o model.
    pub const OPENAI_GPT_4O: &str = "gpt-4o";
    /// Constant for the GPT-5 model.
    pub const OPENAI_GPT_5: &str = "gpt-5";
    /// Constant for the GPT-5 Pro model.
    pub const OPENAI_GPT_5_PRO: &str = "gpt-5-pro";
    /// Constant for the O3 model.
    pub const OPENAI_O3: &str = "o3";
    /// Constant for the O4 Mini model.
    pub const OPENAI_O4_MINI: &str = "o4-mini";

    // Google Gemini models (fully compatible via official compatibility layer)
    /// Constant for the Gemini 2.5 Pro model.
    pub const GOOGLE_GEMINI_2_5_PRO: &str = "gemini-2.5-pro";
    /// Constant for the Gemini 2.5 Flash model.
    pub const GOOGLE_GEMINI_2_5_FLASH: &str = "gemini-2.5-flash";
    /// Constant for the Gemini 2.5 Flash Lite model.
    pub const GOOGLE_GEMINI_2_5_FLASH_LITE: &str = "gemini-2.5-flash-lite";

    // Gemini 3 series - Advanced reasoning models with thinking capabilities
    /// Constant for the Gemini 3 Pro model with advanced reasoning.
    pub const GOOGLE_GEMINI_3_PRO: &str = "gemini-3-pro-preview";
    /// Constant for the Gemini 3 Flash model with thinking capabilities.
    pub const GOOGLE_GEMINI_3_FLASH: &str = "gemini-3-flash-preview";
    /// Constant for the Gemini 3 Pro Image model with multimodal reasoning.
    pub const GOOGLE_GEMINI_3_PRO_IMAGE: &str = "gemini-3-pro-image-preview";

    // Groq models (fully compatible)
    /// Constant for the Llama 3.1 8B Instant model.
    pub const GROQ_LLAMA_3_1_8B_INSTANT: &str = "llama-3.1-8b-instant";
    /// Constant for the Llama 3.3 70B Versatile model.
    pub const GROQ_LLAMA_3_3_70B_VERSATILE: &str = "llama-3.3-70b-versatile";
    /// Constant for the moonshotai/kimi-k2-instruct-0905 Instant model.
    pub const KIMI_K2_0925: &str = "moonshotai/kimi-k2-instruct-0905";

    // Cerebras models (fully compatible)
    /// Constant for the Llama3.1 8B model.
    pub const CEREBRAS_LLAMA3_1_8B: &str = "cerebras/llama3.1-8b";

    // Enosis Labs models (fully compatible)
    /// Constant for the Astronomer 1 model.
    pub const ASTRONOMER_1: &str = "astronomer-1";
    /// Constant for the Astronomer 1 Max model.
    pub const ASTRONOMER_1_MAX: &str = "astronomer-1-max";
    /// Constant for the Astronomer 1.5 model.
    pub const ASTRONOMER_1_5: &str = "astronomer-1.5";
    /// Constant for the Astronomer 2 model.
    pub const ASTRONOMER_2: &str = "astronomer-2";
    /// Constant for the Astronomer 2 Pro model.
    pub const ASTRONOMER_2_PRO: &str = "astronomer-2-pro";

    // Legacy aliases for backward compatibility (deprecated - use provider-prefixed versions above)
    /// Legacy constant for the GPT-4o model (use OPENAI_GPT_4O instead).
    #[deprecated(note = "Use OPENAI_GPT_4O instead for OpenAI compatibility")]
    pub const GPT_4O: &str = "openai/gpt-4o";
    /// Legacy constant for the GPT-5 model (use OPENAI_GPT_5 instead).
    #[deprecated(note = "Use OPENAI_GPT_5 instead for OpenAI compatibility")]
    pub const GPT_5: &str = "openai/gpt-5";
    /// Legacy constant for the Gemini 2.5 Pro model (use GOOGLE_GEMINI_2_5_PRO instead).
    #[deprecated(note = "Use GOOGLE_GEMINI_2_5_PRO instead for OpenAI compatibility")]
    pub const GEMINI_2_5_PRO: &str = "google/gemini-2.5-pro";
    /// Legacy constant for the Gemini 2.5 Flash model (use GOOGLE_GEMINI_2_5_FLASH instead).
    #[deprecated(note = "Use GOOGLE_GEMINI_2_5_FLASH instead for OpenAI compatibility")]
    pub const GEMINI_2_5_FLASH: &str = "google/gemini-2.5-flash";
    /// Legacy constant for the Gemini 2.5 Flash Lite model (use GOOGLE_GEMINI_2_5_FLASH_LITE instead).
    #[deprecated(note = "Use GOOGLE_GEMINI_2_5_FLASH_LITE instead for OpenAI compatibility")]
    pub const GEMINI_2_5_FLASH_LITE: &str = "google/gemini-2.5-flash-lite";
    /// Legacy constant for the Llama 3.1 8B Instant model (use GROQ_LLAMA_3_1_8B_INSTANT instead).
    #[deprecated(note = "Use GROQ_LLAMA_3_1_8B_INSTANT instead for OpenAI compatibility")]
    pub const LLAMA_3_1_8B_INSTANT: &str = "groq/llama-3.1-8b-instant";
    /// Legacy constant for the Llama3.1 8B model (use CEREBRAS_LLAMA3_1_8B instead).
    #[deprecated(note = "Use CEREBRAS_LLAMA3_1_8B instead for OpenAI compatibility")]
    pub const LLAMA3_1_8B: &str = "cerebras/llama3.1-8b";
}

/// A collection of predefined provider name constants for convenience.
pub mod providers {
    /// Constant for the OpenAI provider.
    pub const OPENAI: &str = "openai";
    /// Constant for the Anthropic provider.
    pub const ANTHROPIC: &str = "anthropic";
    /// Constant for the Groq provider.
    pub const GROQ: &str = "groq";
    /// Constant for the Cerebras provider.
    pub const CEREBRAS: &str = "cerebras";
    /// Constant for the Gemini provider.
    pub const GEMINI: &str = "gemini";
    /// Constant for the Enosis Labs provider.
    pub const ENOSISLABS: &str = "enosislabs";
}

impl ChatCompletionRequest {
    /// Creates a new `ChatCompletionRequest` with the given model and messages.
    ///
    /// # Arguments
    ///
    /// * `model` - The identifier of the model to use.
    /// * `messages` - The list of messages for the conversation.
    pub fn new(model: impl Into<String>, messages: Vec<ChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            user: None,
            provider: None,
            stream: None,
            logit_bias: None,
            logprobs: None,
            top_logprobs: None,
            n: None,
            response_format: None,
            tools: None,
            tool_choice: None,
            thinking_config: None,
        }
    }

    /// Sets the temperature for the chat completion.
    ///
    /// The temperature is clamped between 0.0 and 2.0.
    ///
    /// # Arguments
    ///
    /// * `temperature` - The sampling temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature.clamp(0.0, 2.0));
        self
    }

    /// Sets the maximum number of tokens to generate.
    ///
    /// # Arguments
    ///
    /// * `max_tokens` - The maximum number of tokens.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the user identifier for the chat completion.
    ///
    /// # Arguments
    ///
    /// * `user` - A unique identifier for the end-user.
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Sets a provider hint for the request.
    ///
    /// # Arguments
    ///
    /// * `provider` - The name of the provider to use.
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// Enables or disables streaming for the response.
    ///
    /// # Arguments
    ///
    /// * `stream` - `true` to enable streaming, `false` to disable.
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Sets the logit bias for the chat completion.
    ///
    /// # Arguments
    ///
    /// * `logit_bias` - A map of token IDs to bias values.
    pub fn with_logit_bias(mut self, logit_bias: serde_json::Value) -> Self {
        self.logit_bias = Some(logit_bias);
        self
    }

    /// Enables or disables log probabilities for the response.
    ///
    /// # Arguments
    ///
    /// * `logprobs` - `true` to include log probabilities.
    pub fn with_logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = Some(logprobs);
        self
    }

    /// Sets the number of most likely tokens to return at each position.
    ///
    /// # Arguments
    ///
    /// * `top_logprobs` - The number of top log probabilities to return.
    pub fn with_top_logprobs(mut self, top_logprobs: u32) -> Self {
        self.top_logprobs = Some(top_logprobs);
        self
    }

    /// Sets the number of chat completion choices to generate.
    ///
    /// # Arguments
    ///
    /// * `n` - The number of completions to generate.
    pub fn with_n(mut self, n: u32) -> Self {
        self.n = Some(n);
        self
    }

    /// Sets the response format for the chat completion.
    ///
    /// # Arguments
    ///
    /// * `response_format` - The format the model must output.
    pub fn with_response_format(mut self, response_format: ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }

    /// Sets the tools available to the model.
    ///
    /// # Arguments
    ///
    /// * `tools` - A list of tools the model can use.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Sets the tool choice for the chat completion.
    ///
    /// # Arguments
    ///
    /// * `tool_choice` - Controls which tool the model uses.
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the thinking configuration for Gemini 3 and 2.5 series models.
    ///
    /// # Arguments
    ///
    /// * `thinking_config` - Configuration for thinking capabilities.
    pub fn with_thinking_config(mut self, thinking_config: ThinkingConfig) -> Self {
        self.thinking_config = Some(thinking_config);
        self
    }

    /// Enables thought summaries in the response (Gemini 3 and 2.5 series).
    ///
    /// # Arguments
    ///
    /// * `include_thoughts` - Whether to include thought summaries.
    pub fn with_include_thoughts(mut self, include_thoughts: bool) -> Self {
        let mut config = self.thinking_config.unwrap_or_default();
        config.include_thoughts = Some(include_thoughts);
        self.thinking_config = Some(config);
        self
    }

    /// Sets the thinking level for Gemini 3 models.
    ///
    /// # Arguments
    ///
    /// * `thinking_level` - The thinking level (minimal, low, medium, high).
    pub fn with_thinking_level(mut self, thinking_level: ThinkingLevel) -> Self {
        let mut config = self.thinking_config.unwrap_or_default();
        config.thinking_level = Some(thinking_level);
        self.thinking_config = Some(config);
        self
    }

    /// Sets the thinking budget for Gemini 2.5 models.
    ///
    /// # Arguments
    ///
    /// * `thinking_budget` - Number of thinking tokens (-1 for dynamic, 0 to disable).
    pub fn with_thinking_budget(mut self, thinking_budget: i32) -> Self {
        let mut config = self.thinking_config.unwrap_or_default();
        config.thinking_budget = Some(thinking_budget);
        self.thinking_config = Some(config);
        self
    }

    /// Validates that the request parameters are compatible with OpenAI standards.
    ///
    /// This method checks parameter ranges and values to ensure they match OpenAI's API specifications.
    /// Also validates Gemini 3 specific parameters like thinking configuration.
    ///
    /// # Returns
    ///
    /// A `Result` indicating whether the request is valid for OpenAI compatibility.
    pub fn validate_openai_compatibility(&self) -> Result<(), String> {
        // Validate temperature
        if let Some(temp) = self.temperature {
            if !(0.0..=2.0).contains(&temp) {
                return Err(format!(
                    "Temperature must be between 0.0 and 2.0, got {}",
                    temp
                ));
            }
        }

        // Validate top_p
        if let Some(top_p) = self.top_p {
            if !(0.0..=1.0).contains(&top_p) {
                return Err(format!("Top-p must be between 0.0 and 1.0, got {}", top_p));
            }
        }

        // Validate frequency_penalty
        if let Some(fp) = self.frequency_penalty {
            if !(-2.0..=2.0).contains(&fp) {
                return Err(format!(
                    "Frequency penalty must be between -2.0 and 2.0, got {}",
                    fp
                ));
            }
        }

        // Validate presence_penalty
        if let Some(pp) = self.presence_penalty {
            if !(-2.0..=2.0).contains(&pp) {
                return Err(format!(
                    "Presence penalty must be between -2.0 and 2.0, got {}",
                    pp
                ));
            }
        }

        // Validate max_tokens
        if let Some(mt) = self.max_tokens {
            if mt == 0 {
                return Err("Max tokens must be greater than 0".to_string());
            }
        }

        // Validate top_logprobs
        if let Some(tlp) = self.top_logprobs {
            if !(0..=20).contains(&tlp) {
                return Err(format!(
                    "Top logprobs must be between 0 and 20, got {}",
                    tlp
                ));
            }
        }

        // Validate n
        if let Some(n) = self.n {
            if n == 0 {
                return Err("n must be greater than 0".to_string());
            }
        }

        // Validate stop sequences
        if let Some(stop) = &self.stop {
            if stop.len() > 4 {
                return Err("Cannot have more than 4 stop sequences".to_string());
            }
            for seq in stop {
                if seq.is_empty() {
                    return Err("Stop sequences cannot be empty".to_string());
                }
                if seq.len() > 64 {
                    return Err("Stop sequences cannot be longer than 64 characters".to_string());
                }
            }
        }

        // Validate thinking configuration for Gemini models
        if let Some(thinking_config) = &self.thinking_config {
            self.validate_thinking_config(thinking_config)?;
        }

        Ok(())
    }

    /// Validates thinking configuration parameters for Gemini models.
    fn validate_thinking_config(&self, config: &ThinkingConfig) -> Result<(), String> {
        let is_gemini_3 = self.model.contains("gemini-3");
        let is_gemini_2_5 = self.model.contains("gemini-2.5");
        let is_gemini_3_pro = self.model.contains("gemini-3-pro");

        // Validate thinking level (Gemini 3 only)
        if let Some(level) = &config.thinking_level {
            if !is_gemini_3 {
                return Err("thinking_level is only supported for Gemini 3 models".to_string());
            }

            match level {
                ThinkingLevel::Minimal | ThinkingLevel::Medium => {
                    if is_gemini_3_pro {
                        return Err(
                            "Gemini 3 Pro only supports 'low' and 'high' thinking levels"
                                .to_string(),
                        );
                    }
                }
                _ => {}
            }
        }

        // Validate thinking budget (Gemini 2.5 only)
        if let Some(budget) = config.thinking_budget {
            if !is_gemini_2_5 {
                return Err("thinking_budget is only supported for Gemini 2.5 models".to_string());
            }

            // Validate budget ranges based on model
            if self.model.contains("2.5-pro") {
                if budget != -1 && !(128..=32768).contains(&budget) {
                    return Err(
                        "Gemini 2.5 Pro thinking budget must be -1 (dynamic) or between 128-32768"
                            .to_string(),
                    );
                }
            } else if self.model.contains("2.5-flash")
                && budget != -1
                && !(0..=24576).contains(&budget)
            {
                return Err(
                    "Gemini 2.5 Flash thinking budget must be -1 (dynamic) or between 0-24576"
                        .to_string(),
                );
            }
        }

        // Warn about conflicting parameters
        if config.thinking_level.is_some() && config.thinking_budget.is_some() {
            return Err("Cannot specify both thinking_level (Gemini 3) and thinking_budget (Gemini 2.5) in the same request".to_string());
        }

        Ok(())
    }

    /// Checks if the model supports thinking capabilities.
    pub fn supports_thinking(&self) -> bool {
        self.model.contains("gemini-3") || self.model.contains("gemini-2.5")
    }

    /// Checks if the model requires thought signatures for function calling.
    pub fn requires_thought_signatures(&self) -> bool {
        self.model.contains("gemini-3")
    }
}

impl OpenAIChatCompletionRequest {
    /// Creates a new OpenAI-compatible chat completion request.
    pub fn new(model: impl Into<String>, messages: Vec<OpenAIChatMessage>) -> Self {
        Self {
            model: model.into(),
            messages,
            temperature: None,
            max_tokens: None,
            top_p: None,
            frequency_penalty: None,
            presence_penalty: None,
            stop: None,
            user: None,
            provider: None,
            stream: None,
            logit_bias: None,
            logprobs: None,
            top_logprobs: None,
            n: None,
            response_format: None,
            tools: None,
            tool_choice: None,
            thinking_config: None,
            thinking: None,
        }
    }

    /// Sets the sampling temperature.
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature.clamp(0.0, 2.0));
        self
    }

    /// Sets the maximum number of tokens to generate.
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }

    /// Sets the end-user identifier.
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }

    /// Sets a provider hint.
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }

    /// Enables or disables streaming.
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }

    /// Sets nucleus sampling.
    pub fn with_top_p(mut self, top_p: f32) -> Self {
        self.top_p = Some(top_p.clamp(0.0, 1.0));
        self
    }

    /// Sets frequency penalty.
    pub fn with_frequency_penalty(mut self, frequency_penalty: f32) -> Self {
        self.frequency_penalty = Some(frequency_penalty.clamp(-2.0, 2.0));
        self
    }

    /// Sets presence penalty.
    pub fn with_presence_penalty(mut self, presence_penalty: f32) -> Self {
        self.presence_penalty = Some(presence_penalty.clamp(-2.0, 2.0));
        self
    }

    /// Sets stop sequences.
    pub fn with_stop(mut self, stop: Vec<String>) -> Self {
        self.stop = Some(stop);
        self
    }

    /// Sets logit bias.
    pub fn with_logit_bias(mut self, logit_bias: serde_json::Value) -> Self {
        self.logit_bias = Some(logit_bias);
        self
    }

    /// Enables or disables log probabilities.
    pub fn with_logprobs(mut self, logprobs: bool) -> Self {
        self.logprobs = Some(logprobs);
        self
    }

    /// Sets the top log probabilities count.
    pub fn with_top_logprobs(mut self, top_logprobs: u32) -> Self {
        self.top_logprobs = Some(top_logprobs);
        self
    }

    /// Sets the number of choices to generate.
    pub fn with_n(mut self, n: u32) -> Self {
        self.n = Some(n);
        self
    }

    /// Sets the response format.
    pub fn with_response_format(mut self, response_format: ResponseFormat) -> Self {
        self.response_format = Some(response_format);
        self
    }

    /// Sets the available tools.
    pub fn with_tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = Some(tools);
        self
    }

    /// Sets the tool choice strategy.
    pub fn with_tool_choice(mut self, tool_choice: ToolChoice) -> Self {
        self.tool_choice = Some(tool_choice);
        self
    }

    /// Sets the Gemini thinking configuration.
    pub fn with_thinking_config(mut self, thinking_config: ThinkingConfig) -> Self {
        self.thinking_config = Some(thinking_config);
        self
    }

    /// Enables or disables thought summaries.
    pub fn with_include_thoughts(mut self, include_thoughts: bool) -> Self {
        let mut config = self.thinking_config.unwrap_or_default();
        config.include_thoughts = Some(include_thoughts);
        self.thinking_config = Some(config);
        self
    }

    /// Sets the Gemini 3 thinking level.
    pub fn with_thinking_level(mut self, thinking_level: ThinkingLevel) -> Self {
        let mut config = self.thinking_config.unwrap_or_default();
        config.thinking_level = Some(thinking_level);
        self.thinking_config = Some(config);
        self
    }

    /// Sets the Gemini 2.5 thinking budget.
    pub fn with_thinking_budget(mut self, thinking_budget: i32) -> Self {
        let mut config = self.thinking_config.unwrap_or_default();
        config.thinking_budget = Some(thinking_budget);
        self.thinking_config = Some(config);
        self
    }

    /// Sets Anthropic extended-thinking configuration.
    ///
    /// Serialised as `thinking: {"type":"enabled","budget_tokens":N}` — the format
    /// expected by Anthropic's API via OpenRouter/Rainy API.
    pub fn with_anthropic_thinking(mut self, budget_tokens: i32) -> Self {
        self.thinking =
            Some(serde_json::json!({"type": "enabled", "budget_tokens": budget_tokens}));
        self
    }

    /// Validates compatibility using the same parameter rules as the simple chat request.
    pub fn validate_openai_compatibility(&self) -> Result<(), String> {
        ChatCompletionRequest {
            model: self.model.clone(),
            messages: vec![],
            temperature: self.temperature,
            max_tokens: self.max_tokens,
            top_p: self.top_p,
            frequency_penalty: self.frequency_penalty,
            presence_penalty: self.presence_penalty,
            stop: self.stop.clone(),
            user: self.user.clone(),
            provider: self.provider.clone(),
            stream: self.stream,
            logit_bias: self.logit_bias.clone(),
            logprobs: self.logprobs,
            top_logprobs: self.top_logprobs,
            n: self.n,
            response_format: self.response_format.clone(),
            tools: self.tools.clone(),
            tool_choice: self.tool_choice.clone(),
            thinking_config: self.thinking_config.clone(),
        }
        .validate_openai_compatibility()
    }

    /// Checks whether the selected model supports thinking features.
    pub fn supports_thinking(&self) -> bool {
        self.model.contains("gemini-3") || self.model.contains("gemini-2.5")
    }

    /// Checks whether the selected model requires thought signatures for function calling.
    pub fn requires_thought_signatures(&self) -> bool {
        self.model.contains("gemini-3")
    }
}

impl ChatMessage {
    /// Creates a new message with the `System` role.
    ///
    /// # Arguments
    ///
    /// * `content` - The content of the system message.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }

    /// Creates a new message with the `User` role.
    ///
    /// # Arguments
    ///
    /// * `content` - The content of the user message.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }

    /// Creates a new message with the `Assistant` role.
    ///
    /// # Arguments
    ///
    /// * `content` - The content of the assistant message.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

impl OpenAIMessageContent {
    /// Creates text content.
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text(content.into())
    }

    /// Creates multimodal content parts.
    pub fn parts(parts: Vec<OpenAIContentPart>) -> Self {
        Self::Parts(parts)
    }
}

impl OpenAIContentPart {
    /// Creates a text content part.
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

    /// Creates an image URL part with a specific detail hint.
    pub fn image_url_with_detail(url: impl Into<String>, detail: impl Into<String>) -> Self {
        Self::ImageUrl {
            image_url: OpenAIImageUrl {
                url: url.into(),
                detail: Some(detail.into()),
            },
        }
    }
}

impl OpenAIChatMessage {
    /// Creates a new system message.
    pub fn system(content: impl Into<OpenAIMessageContent>) -> Self {
        Self {
            role: OpenAIMessageRole::System,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Creates a new user message.
    pub fn user(content: impl Into<OpenAIMessageContent>) -> Self {
        Self {
            role: OpenAIMessageRole::User,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Creates a new assistant message.
    pub fn assistant(content: impl Into<OpenAIMessageContent>) -> Self {
        Self {
            role: OpenAIMessageRole::Assistant,
            content: Some(content.into()),
            name: None,
            tool_calls: None,
            tool_call_id: None,
        }
    }

    /// Creates an assistant message that only carries tool calls.
    pub fn assistant_with_tool_calls(tool_calls: Vec<OpenAIToolCall>) -> Self {
        Self {
            role: OpenAIMessageRole::Assistant,
            content: None,
            name: None,
            tool_calls: Some(tool_calls),
            tool_call_id: None,
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
        }
    }

    /// Creates a message with full control over optional OpenAI-compatible fields.
    pub fn with_parts(
        role: OpenAIMessageRole,
        content: Option<OpenAIMessageContent>,
        tool_calls: Option<Vec<OpenAIToolCall>>,
        tool_call_id: Option<String>,
    ) -> Self {
        Self {
            role,
            content,
            name: None,
            tool_calls,
            tool_call_id,
        }
    }
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

// Legacy compatibility types - keep existing types for backward compatibility
use uuid::Uuid;

/// Represents a user account (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    /// The unique ID of the user.
    pub id: Uuid,
    /// The user's identifier string.
    pub user_id: String,
    /// The name of the user's subscription plan.
    pub plan_name: String,
    /// The user's current credit balance.
    pub current_credits: f64,
    /// The amount of credits the user has used in the current month.
    pub credits_used_this_month: f64,
    /// The date when the user's credits will reset.
    pub credits_reset_date: DateTime<Utc>,
    /// Indicates if the user account is active.
    pub is_active: bool,
    /// The timestamp of when the user account was created.
    pub created_at: DateTime<Utc>,
}

/// Represents an API key (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// The unique ID of the API key.
    pub id: Uuid,
    /// The API key string.
    pub key: String,
    /// The ID of the user who owns the key.
    pub owner_id: Uuid,
    /// Indicates if the API key is active.
    pub is_active: bool,
    /// The timestamp of when the key was created.
    pub created_at: DateTime<Utc>,
    /// The expiration date of the key, if any.
    pub expires_at: Option<DateTime<Utc>>,
    /// A description of the key.
    pub description: Option<String>,
    /// The timestamp of when the key was last used.
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Represents usage statistics over a period (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    /// The number of days in the usage period.
    pub period_days: u32,
    /// A list of daily usage data.
    pub daily_usage: Vec<DailyUsage>,
    /// A list of recent credit transactions.
    pub recent_transactions: Vec<CreditTransaction>,
    /// The total number of requests made in the period.
    pub total_requests: u64,
    /// The total number of tokens used in the period.
    pub total_tokens: u64,
}

/// Represents usage data for a single day (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    /// The date for the usage data.
    pub date: String,
    /// The number of credits used on this day.
    pub credits_used: f64,
    /// The number of requests made on this day.
    pub requests: u64,
    /// The number of tokens used on this day.
    pub tokens: u64,
}

/// Represents a single credit transaction (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditTransaction {
    /// The unique ID of the transaction.
    pub id: Uuid,
    /// The type of the transaction.
    pub transaction_type: TransactionType,
    /// The amount of credits involved in the transaction.
    pub credits_amount: f64,
    /// The credit balance after the transaction.
    pub credits_balance_after: f64,
    /// The provider associated with the transaction, if any.
    pub provider: Option<String>,
    /// The model associated with the transaction, if any.
    pub model: Option<String>,
    /// A description of the transaction.
    pub description: String,
    /// The timestamp of when the transaction occurred.
    pub created_at: DateTime<Utc>,
}

/// The type of credit transaction (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    /// A transaction for API usage.
    Usage,
    /// A transaction for a credit reset.
    Reset,
    /// A transaction for a credit purchase.
    Purchase,
    /// A transaction for a credit refund.
    Refund,
}

// Legacy aliases for backward compatibility
/// A legacy type alias for `MessageRole`.
pub type ChatRole = MessageRole;
/// A legacy type alias for `Usage`.
pub type ChatUsage = Usage;
/// A legacy type alias for `HealthStatus`.
pub type HealthCheck = HealthStatus;

/// Represents the status of backend services (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthServices {
    /// The status of the database connection.
    pub database: bool,
    /// The status of the Redis connection.
    pub redis: bool,
    /// The overall status of AI providers.
    pub providers: bool,
}

/// The health status of the API (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatusEnum {
    /// The API is healthy.
    Healthy,
    /// The API is in a degraded state.
    Degraded,
    /// The API is unhealthy.
    Unhealthy,
    /// The API needs initialization.
    NeedsInit,
}

/// Represents the format that the model must output.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResponseFormat {
    /// The model can return text.
    Text,
    /// The model must return a valid JSON object.
    JsonObject,
    /// The model must return a JSON object that matches the provided schema.
    JsonSchema {
        /// The JSON Schema that the model's output must conform to.
        json_schema: serde_json::Value,
    },
}

/// Represents a tool that the model can use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// The type of the tool (currently only "function" is supported).
    pub r#type: ToolType,
    /// The function definition describing the tool's capabilities.
    pub function: FunctionDefinition,
}

/// The type of tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    /// A function tool.
    Function,
}

/// Represents a function definition for a tool.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionDefinition {
    /// The name of the function.
    pub name: String,
    /// A description of what the function does.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// The parameters the function accepts, described as a JSON Schema object.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<serde_json::Value>,
}

/// Controls which tool is called by the model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ToolChoice {
    /// No tool is called.
    None,
    /// The model chooses which tool to call.
    Auto,
    /// A specific tool is called.
    Tool {
        /// The type of the tool being called.
        r#type: ToolType,
        /// The function to call within the tool.
        function: ToolFunction,
    },
}

/// Represents a tool function call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolFunction {
    /// The name of the function to call.
    pub name: String,
}

/// Configuration for thinking capabilities in Gemini 3 and 2.5 series models.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ThinkingConfig {
    /// Whether to include thought summaries in the response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_thoughts: Option<bool>,

    /// The thinking level for Gemini 3 models (low, high for Pro; minimal, low, medium, high for Flash).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_level: Option<ThinkingLevel>,

    /// The thinking budget for Gemini 2.5 models (number of thinking tokens).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thinking_budget: Option<i32>,
}

/// Thinking levels for Gemini 3 models.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ThinkingLevel {
    /// Minimal thinking (Gemini 3 Flash only) - model likely won't think.
    Minimal,
    /// Low thinking level - faster responses with basic reasoning.
    Low,
    /// Medium thinking level (Gemini 3 Flash only) - balanced reasoning and speed.
    Medium,
    /// High thinking level - deep reasoning for complex tasks (default).
    High,
}

/// Represents a content part that may include thought signatures.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ContentPart {
    /// The text content of the part.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,

    /// Function call information if this part contains a function call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_call: Option<FunctionCall>,

    /// Function response information if this part contains a function response.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function_response: Option<FunctionResponse>,

    /// Indicates if this part contains thought content.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<bool>,

    /// Encrypted thought signature for preserving reasoning context across turns.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought_signature: Option<String>,
}

/// Represents a function call in the content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionCall {
    /// The name of the function being called.
    pub name: String,
    /// The arguments for the function call as a JSON object.
    pub args: serde_json::Value,
}

/// Represents a function response in the content.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionResponse {
    /// The name of the function that was called.
    pub name: String,
    /// The response from the function call.
    pub response: serde_json::Value,
}

/// Enhanced chat message that supports Gemini 3 thinking capabilities.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EnhancedChatMessage {
    /// The role of the message author.
    pub role: MessageRole,
    /// The content parts of the message (supports text, function calls, and thought signatures).
    pub parts: Vec<ContentPart>,
}

/// Enhanced usage statistics that include thinking tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedUsage {
    /// The number of tokens in the prompt.
    pub prompt_tokens: u32,
    /// The number of tokens in the generated completion.
    pub completion_tokens: u32,
    /// The total number of tokens used in the request (prompt + completion).
    pub total_tokens: u32,
    /// The number of thinking tokens used (Gemini 3 and 2.5 series).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thoughts_token_count: Option<u32>,
}

impl ThinkingConfig {
    /// Creates a new thinking configuration with default values.
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates a configuration for Gemini 3 models with specified thinking level.
    ///
    /// # Arguments
    ///
    /// * `level` - The thinking level to use.
    /// * `include_thoughts` - Whether to include thought summaries.
    pub fn gemini_3(level: ThinkingLevel, include_thoughts: bool) -> Self {
        Self {
            thinking_level: Some(level),
            include_thoughts: Some(include_thoughts),
            thinking_budget: None,
        }
    }

    /// Creates a configuration for Gemini 2.5 models with specified thinking budget.
    ///
    /// # Arguments
    ///
    /// * `budget` - The thinking budget (-1 for dynamic, 0 to disable, or specific token count).
    /// * `include_thoughts` - Whether to include thought summaries.
    pub fn gemini_2_5(budget: i32, include_thoughts: bool) -> Self {
        Self {
            thinking_budget: Some(budget),
            include_thoughts: Some(include_thoughts),
            thinking_level: None,
        }
    }

    /// Creates a configuration optimized for complex reasoning tasks.
    pub fn high_reasoning() -> Self {
        Self {
            thinking_level: Some(ThinkingLevel::High),
            include_thoughts: Some(true),
            thinking_budget: Some(-1), // Dynamic for 2.5 models
        }
    }

    /// Creates a configuration optimized for fast responses.
    pub fn fast_response() -> Self {
        Self {
            thinking_level: Some(ThinkingLevel::Low),
            include_thoughts: Some(false),
            thinking_budget: Some(512), // Low budget for 2.5 models
        }
    }
}

impl ContentPart {
    /// Creates a new text content part.
    pub fn text(content: impl Into<String>) -> Self {
        Self {
            text: Some(content.into()),
            function_call: None,
            function_response: None,
            thought: None,
            thought_signature: None,
        }
    }

    /// Creates a new function call content part.
    pub fn function_call(name: impl Into<String>, args: serde_json::Value) -> Self {
        Self {
            text: None,
            function_call: Some(FunctionCall {
                name: name.into(),
                args,
            }),
            function_response: None,
            thought: None,
            thought_signature: None,
        }
    }

    /// Creates a new function response content part.
    pub fn function_response(name: impl Into<String>, response: serde_json::Value) -> Self {
        Self {
            text: None,
            function_call: None,
            function_response: Some(FunctionResponse {
                name: name.into(),
                response,
            }),
            thought: None,
            thought_signature: None,
        }
    }

    /// Adds a thought signature to this content part.
    pub fn with_thought_signature(mut self, signature: impl Into<String>) -> Self {
        self.thought_signature = Some(signature.into());
        self
    }

    /// Marks this content part as containing thought content.
    pub fn as_thought(mut self) -> Self {
        self.thought = Some(true);
        self
    }
}

impl EnhancedChatMessage {
    /// Creates a new enhanced message with the `System` role.
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            parts: vec![ContentPart::text(content)],
        }
    }

    /// Creates a new enhanced message with the `User` role.
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            parts: vec![ContentPart::text(content)],
        }
    }

    /// Creates a new enhanced message with the `Assistant` role.
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            parts: vec![ContentPart::text(content)],
        }
    }

    /// Creates a new enhanced message with multiple content parts.
    pub fn with_parts(role: MessageRole, parts: Vec<ContentPart>) -> Self {
        Self { role, parts }
    }
}

/// Represents a streaming chat completion response (OpenAI delta format).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionStreamResponse {
    /// A unique identifier for the chat completion.
    pub id: String,
    /// The type of object, which is always "chat.completion.chunk".
    pub object: String,
    /// The Unix timestamp (in seconds) of when the completion was created.
    pub created: u64,
    /// The model that was used for the completion.
    pub model: String,
    /// A list of chat completion choices.
    pub choices: Vec<ChatCompletionStreamChoice>,
    /// Information about the token usage for this completion (only present in the final chunk).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Represents a single choice in a streaming chat completion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionStreamChoice {
    /// The index of the choice in the list of choices.
    pub index: u32,
    /// The delta containing the new content for this choice.
    pub delta: ChatCompletionStreamDelta,
    /// The reason the model stopped generating tokens (only present in the final chunk).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub finish_reason: Option<String>,
}

/// Represents the delta (change) in a streaming chat completion response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionStreamDelta {
    /// The role of the message (only present in the first chunk).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    /// The new content for this chunk.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    /// The thinking/reasoning content for this chunk (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub thought: Option<String>,
    /// Tool calls for this chunk (if any).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,
}

/// Represents a tool call in a streaming response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCall {
    /// The index of the tool call.
    pub index: u32,
    /// The ID of the tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    /// The type of the tool call.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    /// The function being called.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub function: Option<ToolCallFunction>,
}

/// Represents a function call in a tool call.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallFunction {
    /// The name of the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The arguments for the function.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
}
