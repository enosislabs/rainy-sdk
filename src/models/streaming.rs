//! Streaming domain facade.

/// Stream types remain re-exported from the compatibility root while this
/// module provides a stable domain import path.
pub use super::{
    ChatCompletionStreamChoice, ChatCompletionStreamDelta, ChatCompletionStreamResponse,
    ChatStreamEvent, RainyBillingStreamEvent, RainyBillingUsage, ResponsesStreamEvent, ToolCall,
    ToolCallFunction,
};
