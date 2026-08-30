//! Public request and response models grouped by wire protocol.
//!
//! The module deliberately follows protocol boundaries instead of mirroring
//! the implementation of any one gateway. Stable fields are typed; provider
//! extensions and genuinely open JSON shapes are preserved in explicit
//! `serde_json::Value` fields or extension maps.

pub mod catalog;
pub mod chat;
pub mod common;
pub mod embeddings;
pub mod messages;
pub mod reasoning;
pub mod responses;
pub mod streaming;
pub mod tools;

#[cfg(feature = "legacy")]
mod legacy_types;

pub use catalog::{
    CapabilityFlag, ModelArchitecture, ModelCatalogItem, ModelList, ModelListItem, ModelPricing,
    ModelSelectionCriteria, RainyCapabilities, ReasoningMode, ReasoningPreference,
    build_reasoning_config, select_models,
};
pub use chat::{
    ChatChoice, ChatCompletionRequest, ChatCompletionResponse, ChatImageConfig, ChatMessage,
    ChatModality, ChatRequest, ChatResponse, ChatStreamOptions, MessageRole, OpenAIChatChoice,
    OpenAIChatCompletionRequest, OpenAIChatCompletionResponse, OpenAIChatMessage,
    OpenAIContentPart, OpenAIFile, OpenAIFunctionCall, OpenAIImageUrl, OpenAIInputAudio,
    OpenAIMessageContent, OpenAIMessageRole, OpenAIToolCall, ProviderOptions, StopSequences,
};
pub use common::{
    AvailableModels, CompatWarning, CreditInfo, FeaturesUsed, HealthStatus, RainyEnvelope,
    RainyEnvelopeMeta, ReasoningMeta, RequestMetadata, ResearchDepth, ResearchProvider,
    ServiceStatus, Usage,
};
pub use embeddings::{
    EmbeddingData, EmbeddingEncodingFormat, EmbeddingInput, EmbeddingValue, EmbeddingsRequest,
    EmbeddingsResponse, EmbeddingsUsage,
};
pub use messages::{
    AnthropicContent, AnthropicContentBlock, AnthropicImageSource, AnthropicMessage,
    AnthropicMessageRequest, AnthropicMessageResponse, AnthropicMessageStreamEvent,
    AnthropicMessageStreamEventType, AnthropicServiceTier, AnthropicThinking, AnthropicTool,
    AnthropicToolChoice, AnthropicUsage,
};
pub use reasoning::{
    ReasoningBudget, ReasoningConfig, ReasoningControl, ReasoningEffort, ReasoningRequest,
};
pub use responses::{
    ResponsesApiResponse, ResponsesEvent, ResponsesEventType, ResponsesInputItem, ResponsesRequest,
    ResponsesStreamEvent, ResponsesUsage,
};
pub use streaming::{
    ChatCompletionStreamChoice, ChatCompletionStreamDelta, ChatCompletionStreamResponse,
    ChatStreamEvent, RainyBillingStreamEvent, RainyBillingUsage, ToolCall, ToolCallFunction,
};
pub use tools::{FunctionDefinition, ResponseFormat, Tool, ToolChoice, ToolFunction, ToolType};

#[cfg(feature = "legacy")]
pub use legacy_types::{
    ApiKey, ChatRole, ChatUsage, CreditTransaction, DailyUsage, HealthCheck, HealthServices,
    HealthStatusEnum, TransactionType, UsageStats, User,
};
