use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableModels {
    /// A map where keys are provider names and values are lists of model names.
    pub providers: HashMap<String, Vec<String>>,

    /// The total number of available models across all providers.
    pub total_models: usize,

    /// A list of provider names that are currently active and available.
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
                && budget != -1 && !(0..=24576).contains(&budget) {
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
    JsonSchema { json_schema: serde_json::Value },
}

/// Represents a tool that the model can use.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    /// The type of the tool.
    pub r#type: ToolType,
    /// The function definition for the tool.
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
        r#type: ToolType,
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
