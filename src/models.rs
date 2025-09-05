use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Chat message with role and content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatMessage {
    pub role: MessageRole,
    pub content: String,
}

/// Message roles supported by the API
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    System,
    User,
    Assistant,
}

/// Chat completion request with enhanced options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    /// Model identifier (e.g., "gpt-4o", "claude-sonnet-4")
    pub model: String,
    
    /// List of messages for the conversation
    pub messages: Vec<ChatMessage>,
    
    /// Sampling temperature (0.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    
    /// Maximum tokens to generate
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    
    /// Nucleus sampling parameter
    #[serde(skip_serializing_if = "Option::is_none")]
    pub top_p: Option<f32>,
    
    /// Frequency penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub frequency_penalty: Option<f32>,
    
    /// Presence penalty (-2.0 to 2.0)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presence_penalty: Option<f32>,
    
    /// Stop sequences
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<Vec<String>>,
    
    /// User identifier for conversation tracking
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
    
    /// Provider hint for model routing
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    
    /// Enable streaming response
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stream: Option<bool>,
}

/// Chat completion response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatCompletionResponse {
    /// Unique identifier for the completion
    pub id: String,
    
    /// Object type (always "chat.completion")
    pub object: String,
    
    /// Unix timestamp of creation
    pub created: u64,
    
    /// Model used for the completion
    pub model: String,
    
    /// List of completion choices
    pub choices: Vec<ChatChoice>,
    
    /// Token usage information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usage: Option<Usage>,
}

/// Individual completion choice
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatChoice {
    /// Choice index
    pub index: u32,
    
    /// Generated message
    pub message: ChatMessage,
    
    /// Reason for completion finish
    pub finish_reason: String,
}

/// Token usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Usage {
    /// Tokens in the prompt
    pub prompt_tokens: u32,
    
    /// Tokens in the completion
    pub completion_tokens: u32,
    
    /// Total tokens used
    pub total_tokens: u32,
}

/// API health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthStatus {
    /// Overall status
    pub status: String,
    
    /// Timestamp of the check
    pub timestamp: String,
    
    /// System uptime in seconds
    pub uptime: f64,
    
    /// Service status details
    pub services: ServiceStatus,
}

/// Individual service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Database connectivity
    pub database: bool,
    
    /// Redis connectivity (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub redis: Option<bool>,
    
    /// AI providers status
    pub providers: bool,
}

/// Available models and providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailableModels {
    /// Map of provider to their models
    pub providers: HashMap<String, Vec<String>>,
    
    /// Total number of available models
    pub total_models: usize,
    
    /// List of active provider names
    pub active_providers: Vec<String>,
}

/// Credit usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditInfo {
    /// Current available credits
    pub current_credits: f64,
    
    /// Estimated cost for the request
    pub estimated_cost: f64,
    
    /// Credits remaining after request
    pub credits_after_request: f64,
    
    /// Next credit reset date
    pub reset_date: String,
}

/// Request metadata from response headers
#[derive(Debug, Clone)]
pub struct RequestMetadata {
    /// Response time in milliseconds
    pub response_time: Option<u64>,
    
    /// Provider that handled the request
    pub provider: Option<String>,
    
    /// Tokens used in the request
    pub tokens_used: Option<u32>,
    
    /// Credits used for the request
    pub credits_used: Option<f64>,
    
    /// Credits remaining after request
    pub credits_remaining: Option<f64>,
    
    /// Request ID for tracking
    pub request_id: Option<String>,
}

/// Predefined model constants for convenience
pub mod models {
    // OpenAI models
    pub const GPT_4O: &str = "gpt-4o";
    pub const GPT_5: &str = "gpt-5";
    pub const GPT_5_PRO: &str = "gpt-5-pro";
    pub const O3: &str = "o3";
    pub const O4_MINI: &str = "o4-mini";
    
    // Anthropic models
    pub const CLAUDE_OPUS_4_1: &str = "claude-opus-4-1";
    pub const CLAUDE_SONNET_4: &str = "claude-sonnet-4";
    
    // Groq models
    pub const LLAMA_3_1_8B_INSTANT: &str = "llama-3.1-8b-instant";
    pub const LLAMA_3_3_70B_VERSATILE: &str = "llama-3.3-70b-versatile";
    pub const DEEPSEEK_R1_DISTILL_LLAMA_70B: &str = "deepseek-r1-distill-llama-70b";
    pub const GROQ_COMPOUND: &str = "groq/compound";
    pub const OPENAI_GPT_OSS_120B: &str = "openai/gpt-oss-120b";
    pub const OPENAI_GPT_OSS_20B: &str = "openai/gpt-oss-20b";
    pub const MOONSHOTAI_KIMI_K2_INSTRUCT: &str = "moonshotai/kimi-k2-instruct";
    pub const QWEN_QWEN3_32B: &str = "qwen/qwen3-32b";
    
    // Cerebras models
    pub const CEREBRAS_OSS_120B: &str = "cerebras-oss-120b";
    pub const QWEN_3_CODER_480B: &str = "qwen-3-coder-480b";
    pub const LLAMA3_1_8B: &str = "llama3.1-8b";
    pub const LLAMA_3_3_70B: &str = "llama-3.3-70b";
    pub const QWEN3_INSTRUCT: &str = "qwen3-instruct";
    
    // Gemini models
    pub const GEMINI_2_5_PRO: &str = "gemini-2.5-pro";
    pub const GEMINI_2_5_FLASH: &str = "gemini-2.5-flash";
    pub const GEMINI_2_5_FLASH_LITE: &str = "gemini-2.5-flash-lite";
}

/// Provider names for convenience
pub mod providers {
    pub const OPENAI: &str = "openai";
    pub const ANTHROPIC: &str = "anthropic";
    pub const GROQ: &str = "groq";
    pub const CEREBRAS: &str = "cerebras";
    pub const GEMINI: &str = "gemini";
}

impl ChatCompletionRequest {
    /// Create a new chat completion request with a model and messages
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
        }
    }
    
    /// Set temperature (0.0 to 2.0)
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = Some(temperature.clamp(0.0, 2.0));
        self
    }
    
    /// Set max tokens
    pub fn with_max_tokens(mut self, max_tokens: u32) -> Self {
        self.max_tokens = Some(max_tokens);
        self
    }
    
    /// Set user identifier for conversation tracking
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }
    
    /// Set provider hint
    pub fn with_provider(mut self, provider: impl Into<String>) -> Self {
        self.provider = Some(provider.into());
        self
    }
    
    /// Enable streaming response
    pub fn with_stream(mut self, stream: bool) -> Self {
        self.stream = Some(stream);
        self
    }
}

impl ChatMessage {
    /// Create a new system message
    pub fn system(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::System,
            content: content.into(),
        }
    }
    
    /// Create a new user message
    pub fn user(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::User,
            content: content.into(),
        }
    }
    
    /// Create a new assistant message
    pub fn assistant(content: impl Into<String>) -> Self {
        Self {
            role: MessageRole::Assistant,
            content: content.into(),
        }
    }
}

// Legacy compatibility types - keep existing types for backward compatibility
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub user_id: String,
    pub plan_name: String,
    pub current_credits: f64,
    pub credits_used_this_month: f64,
    pub credits_reset_date: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub key: String,
    pub owner_id: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub period_days: u32,
    pub daily_usage: Vec<DailyUsage>,
    pub recent_transactions: Vec<CreditTransaction>,
    pub total_requests: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    pub date: String,
    pub credits_used: f64,
    pub requests: u64,
    pub tokens: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditTransaction {
    pub id: Uuid,
    pub transaction_type: TransactionType,
    pub credits_amount: f64,
    pub credits_balance_after: f64,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Usage,
    Reset,
    Purchase,
    Refund,
}

// Legacy aliases for backward compatibility
pub type ChatRole = MessageRole;
pub type ChatUsage = Usage;
pub type HealthCheck = HealthStatus;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthServices {
    pub database: bool,
    pub redis: bool,
    pub providers: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatusEnum {
    Healthy,
    Degraded,
    Unhealthy,
    NeedsInit,
}