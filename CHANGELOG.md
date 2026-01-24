# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.6.0] - 2026-01-23

### 🛡️ Cowork API Integration

Rainy SDK v0.6.0 implements the full **Rainy Cowork API** specifications, including dynamic plan structures, usage tracking, and profile management.

#### Breaking Changes

- **`CoworkPlan` Structure**: Changed from an `enum` to a `struct` to support dynamic plan details returned by the API (id, name, limits).
- **`CoworkCapabilities`**: Refactored to include a nested `CoworkProfile` containing plan and usage info.
- **`CoworkUsage`**: Updated fields to match API response (camelCase mapping).

#### Added

- **`get_cowork_profile()`**: New method to retrieve comprehensive user profile and subscription status.
- **`CoworkProfile`**: New struct mapping the `/cowork/profile` endpoint response.

---

## [0.5.3] - 2026-01-22

### 📚 Documentation Fixes

This release fixes missing documentation that caused CI failures with `cargo doc -D missing-docs`.

#### Fixed

##### Missing Documentation in `cowork.rs`

- Added doc comments to `CoworkPlan` enum variants (`Free`, `GoPlus`, `Plus`, `Pro`, `ProPlus`)
- Added doc comments to `CoworkFeatures` struct fields
- Added doc comments to `CoworkUsage` struct fields
- Added doc comments to `CoworkCapabilities` struct fields

##### Missing Documentation in `models.rs`

- Added doc comment to `ResponseFormat::JsonSchema` variant's `json_schema` field
- Added doc comments to `ToolChoice::Tool` variant's `r#type` and `function` fields

---

## [0.5.2] - 2026-01-22

### 🔑 Cowork API Key Validation

Rainy SDK v0.5.2 introduces **support for Cowork-specific API keys** (`ra-cowork{48 hex}`) used by the Rainy Cowork desktop application.

#### Added

##### New Validation Methods

- **`is_cowork_key()`**: Check if an API key is a Cowork-specific key
  - Returns `true` for keys starting with `ra-cowork`
  - Returns `false` for standard keys starting with `ra-`

##### Enhanced Key Format Validation

- **Standard Keys**: `ra-{48 hex characters}` = 51 characters total
- **Cowork Keys**: `ra-cowork{48 hex characters}` = 57 characters total

#### Changed

##### `AuthConfig::validate()` Improvements

- **Stricter Length Validation**: Now enforces exact key lengths
- **Cowork Detection**: Automatically detects and validates Cowork key format
- **Better Error Messages**: Specific error codes for each key type:
  - `INVALID_API_KEY_FORMAT` for standard key issues
  - `INVALID_COWORK_API_KEY_FORMAT` for Cowork key issues

#### Example

```rust
use rainy_sdk::AuthConfig;

// Standard key (51 chars)
let standard = AuthConfig::new("ra-abc123..."); // 51 chars total
assert!(!standard.is_cowork_key());
assert!(standard.validate().is_ok());

// Cowork key (57 chars)
let cowork = AuthConfig::new("ra-coworkabc123..."); // 57 chars total
assert!(cowork.is_cowork_key());
assert!(cowork.validate().is_ok());
```

#### Technical Details

- **Backward Compatible**: Existing code using valid 51-char keys continues to work
- **Test Coverage**: 37 tests passing (unit, integration, and doc tests)
- **No New Dependencies**: Uses existing validation infrastructure

---

## [0.5.0] - 2025-01-19

### 🚀 Major Feature: Gemini 3 Models with Advanced Thinking Capabilities

Rainy SDK v0.5.0 introduces **complete support for Google's Gemini 3 model family** with advanced thinking capabilities, thought signatures, and enhanced reasoning features.

#### Added

##### New Gemini 3 Models

- **`GOOGLE_GEMINI_3_PRO`**: `gemini-3-pro-preview` - Advanced reasoning with thinking capabilities
- **`GOOGLE_GEMINI_3_FLASH`**: `gemini-3-flash-preview` - Fast, intelligent responses with multimodal support
- **`GOOGLE_GEMINI_3_PRO_IMAGE`**: `gemini-3-pro-image-preview` - Native image generation with contextual understanding

##### Thinking Capabilities (`ThinkingConfig`)

- **`ThinkingLevel`**: Control reasoning depth (Minimal, Low, Medium, High)
  - Gemini 3 Pro: Low, High
  - Gemini 3 Flash: Minimal, Low, Medium, High
- **`thinking_budget`**: Token budget control for Gemini 2.5 models (-1 for dynamic, 0 to disable)
- **`include_thoughts`**: Enable/disable thought summaries in responses
- **Predefined Configurations**:
  - `ThinkingConfig::high_reasoning()` - Optimized for complex tasks
  - `ThinkingConfig::fast_response()` - Optimized for speed
  - `ThinkingConfig::gemini_3()` - Gemini 3 specific configuration
  - `ThinkingConfig::gemini_2_5()` - Gemini 2.5 specific configuration

##### Thought Signatures Support

- **`ContentPart`**: Enhanced content structure supporting:
  - Text content
  - Function calls with arguments
  - Function responses
  - Thought signatures (encrypted reasoning context)
  - Thought markers for reasoning content
- **`EnhancedChatMessage`**: Multi-part messages with thought signature preservation
- **Automatic Validation**: Required for Gemini 3 function calling

##### Enhanced Usage Statistics

- **`EnhancedUsage`**: Extended usage tracking including:
  - `prompt_tokens`: Input token count
  - `completion_tokens`: Output token count
  - `total_tokens`: Combined token count
  - `thoughts_token_count`: Thinking tokens used (Gemini 3 & 2.5)

##### New Request Methods

- **`with_thinking_config()`**: Set complete thinking configuration
- **`with_thinking_level()`**: Set thinking level directly
- **`with_thinking_budget()`**: Set thinking budget for Gemini 2.5
- **`with_include_thoughts()`**: Enable thought summaries
- **`supports_thinking()`**: Check if model supports thinking
- **`requires_thought_signatures()`**: Check if model requires signatures

##### Enhanced Validation

- **`validate_thinking_config()`**: Validate thinking parameters for specific models
- **Gemini 3 Pro Validation**: Ensures only 'low' and 'high' levels are used
- **Gemini 3 Flash Validation**: Supports all four thinking levels
- **Budget Range Validation**: Ensures thinking budgets are within model limits
- **Conflict Detection**: Prevents mixing thinking_level and thinking_budget

##### Content Part Builders

- **`ContentPart::text()`**: Create text content parts
- **`ContentPart::function_call()`**: Create function call parts
- **`ContentPart::function_response()`**: Create function response parts
- **`with_thought_signature()`**: Add encrypted thought signature
- **`as_thought()`**: Mark content as thought/reasoning

##### Enhanced Message Builders

- **`EnhancedChatMessage::system()`**: Create system messages
- **`EnhancedChatMessage::user()`**: Create user messages
- **`EnhancedChatMessage::assistant()`**: Create assistant messages
- **`EnhancedChatMessage::with_parts()`**: Create multi-part messages

#### Improved

##### Model Pricing Information

- **Gemini 3 Flash**: $0.50 input / $3.00 output per 1M tokens (+67%/+20% vs 2.5)
- **Gemini 3 Pro**: $2.00 input / $12.00 output per 1M tokens (+60%/+20% vs 2.5)
- **Gemini 3 Pro Image**: $2.00 input / $12.00 text output / $120.00 image output per 1M tokens

##### Documentation

- **`GEMINI_3_INTEGRATION.md`**: Comprehensive integration guide in Spanish
- **`examples/gemini_3_thinking.rs`**: Complete working examples demonstrating:
  - Complex reasoning with high thinking level
  - Fast responses with low thinking level
  - Function calling with thought signatures
  - Enhanced message format usage
  - Model capability validation
- **Inline Documentation**: Extensive doc comments for all new types and methods

##### Type Safety

- **`FunctionCall`**: Structured function call representation
- **`FunctionResponse`**: Structured function response representation
- **Derive Traits**: Added `PartialEq` to all content-related types for testing

#### Changed

##### Model Constants Organization

- **Gemini 3 Section**: Clearly separated Gemini 3 models with detailed comments
- **Enhanced Documentation**: Each model constant includes pricing and capability notes

##### Request Structure

- **`ChatCompletionRequest`**: Added `thinking_config` optional field
- **Constructor**: Updated to initialize `thinking_config` as `None`
- **Builder Pattern**: Extended with thinking-related methods

#### Technical Details

##### Thinking Token Pricing

- **Gemini 3 Models**: Thinking tokens included in output pricing
- **Cost Calculation**: `output_cost = (completion_tokens + thinking_tokens) × rate`
- **Transparency**: `thoughts_token_count` field provides visibility

##### Thought Signature Requirements

- **Gemini 3 Mandatory**: Thought signatures required for function calling
- **Gemini 2.5 Optional**: Thought signatures recommended but not required
- **Validation**: Automatic validation prevents missing signatures in Gemini 3

##### Compatibility

- **Backward Compatible**: All existing code continues to work
- **Optional Features**: Thinking capabilities are opt-in
- **Graceful Degradation**: Models without thinking support ignore thinking config

#### Examples

##### Basic Thinking Configuration

```rust
let request = ChatCompletionRequest::new(
    GOOGLE_GEMINI_3_PRO,
    vec![ChatMessage::user("Analyze this complex problem...")]
)
.with_thinking_level(ThinkingLevel::High)
.with_include_thoughts(true);
```

##### Function Calling with Thought Signatures

```rust
let message = EnhancedChatMessage::with_parts(
    MessageRole::Assistant,
    vec![
        ContentPart::function_call("get_weather", json!({"city": "Paris"}))
            .with_thought_signature("encrypted_signature_here"),
    ]
);
```

##### Cost Optimization

```rust
// Fast response for simple queries
let fast_config = ThinkingConfig::fast_response();

// Deep reasoning for complex tasks
let deep_config = ThinkingConfig::high_reasoning();
```

#### Migration Guide

##### For Existing Users

No breaking changes - all existing code continues to work. New thinking capabilities are opt-in:

```rust
// Existing code (still works)
let request = ChatCompletionRequest::new(model, messages);

// Enhanced with thinking (optional)
let request = request.with_thinking_level(ThinkingLevel::High);
```

##### For Gemini 2.5 Users

Upgrade to Gemini 3 for enhanced reasoning:

```rust
// Old: Gemini 2.5
model_constants::GOOGLE_GEMINI_2_5_PRO

// New: Gemini 3 with thinking
model_constants::GOOGLE_GEMINI_3_PRO
```

#### Performance Considerations

- **Thinking Tokens**: Gemini 3 models use additional tokens for reasoning
- **Cost Impact**: ~60-67% higher input costs, ~20% higher output costs
- **Quality Improvement**: Significantly better reasoning and problem-solving
- **Optimization**: Use `ThinkingLevel::Low` for simple tasks to reduce costs

#### Acknowledgments

- **Google AI**: For Gemini 3 models and thinking capabilities
- **Google Documentation**: For comprehensive thinking and thought signature guides
- **Community**: For feedback on advanced reasoning requirements

---

## [0.4.0] - 2026-01-18

### 🎯 Major Feature: Cowork Integration

Rainy SDK v0.4.0 introduces **Cowork Integration** - a tier-based feature gating system designed for Rainy Cowork and other client applications. The SDK now acts as the gatekeeper for premium features.

#### Added

##### Cowork Module (`src/cowork.rs`)

- **`CoworkTier`**: Subscription tier enum (Free, Basic, Pro, Enterprise)
- **`CoworkCapabilities`**: Complete capabilities structure including:
  - Available AI models per tier
  - Feature flags
  - Usage limits
  - Validity status
- **`CoworkFeatures`**: Feature flag struct for premium features:
  - `web_research`: Web browsing and research
  - `document_export`: PDF/DOCX export
  - `image_analysis`: AI vision capabilities
  - `automation`: Advanced workflows
  - `priority_queue`: Faster processing
  - `beta_features`: Early access
- **`CoworkLimits`**: Usage limits per tier:
  - `max_tasks_per_day`
  - `max_tokens_per_request`
  - `max_file_size_bytes`

##### New Client Methods

- **`get_cowork_capabilities()`**: Validate API key and retrieve tier info
- **`is_premium()`**: Quick check for premium access
- **`can_use_feature(feature)`**: Check specific feature availability
- **`can_use_model(model)`**: Check model availability for tier
- **`get_cowork_models()`**: Get models available for current tier

##### Tier-Based Model Access

| Tier       | Models                                 |
| ---------- | -------------------------------------- |
| Free       | None (use own Gemini key)              |
| Basic      | GPT-4o, Gemini Flash, Llama 3.1        |
| Pro        | All models including GPT-5, Gemini Pro |
| Enterprise | Full access + beta models              |

#### Changed

- **Feature Flags**: Added `cowork` feature (default enabled)
- **Re-exports**: `CoworkCapabilities`, `CoworkFeatures`, `CoworkLimits`, `CoworkTier` exported at crate root

#### Technical Details

- New endpoint: `/api/v1/cowork/capabilities`
- Graceful fallback to Free tier on network errors
- 4 new unit tests for cowork module

---

## [0.3.0] - 2025-09-22

### 🎯 Major Enhancement: Full OpenAI Compatibility

Rainy SDK v0.3.0 introduces **complete OpenAI API compatibility** while maintaining multi-provider support. This release transforms Rainy SDK into a drop-in replacement for the official OpenAI SDK with enhanced capabilities.

#### Added

##### OpenAI Compatibility Layer

- **100% OpenAI API Compatibility**: All chat completion parameters and responses match OpenAI specifications exactly
- **Drop-in Replacement**: Use Rainy SDK as direct replacement for `async-openai` crate
- **Streaming Compatibility**: OpenAI delta format streaming with tool call support
- **Provider-Prefixed Model Naming**: `openai/gpt-4o`, `google/gemini-2.5-pro`, etc.

##### Advanced OpenAI Parameters

- `logit_bias`: Modify token likelihoods with serde_json::Value support
- `logprobs` & `top_logprobs`: Log probability information for tokens
- `n`: Multiple completion choices generation
- `response_format`: JSON Schema enforcement (Text, JsonObject, JsonSchema)
- `tools` & `tool_choice`: Complete function calling support
- `reasoning_effort`: Control reasoning depth for compatible models

##### Enhanced Model Support

- **Verified Compatible Models Only**:
  - OpenAI: `openai/gpt-4o`, `openai/gpt-5` (native compatibility)
  - Google: `google/gemini-2.5-pro`, `google/gemini-2.5-flash`, `google/gemini-2.5-flash-lite` (via official compatibility layer)
  - Groq: `groq/llama-3.1-8b-instant` (OpenAI-compatible API)
  - Cerebras: `cerebras/llama3.1-8b` (OpenAI-compatible API)
- **Model Discovery API**: `list_available_models()` with real-time compatibility info

##### Parameter Validation

- `validate_openai_compatibility()`: Ensures requests meet OpenAI standards
- Comprehensive validation of parameter ranges and constraints
- Automatic validation for temperature, top_p, penalties, and other parameters

##### Enhanced Streaming

- **OpenAI Delta Format**: Updated `ChatCompletionStreamResponse` with proper delta structure
- **Tool Call Streaming**: Support for streaming function calls
- **Exact SSE Compatibility**: Matches OpenAI's Server-Sent Events format

#### Improved

##### Architecture Enhancements

- **Modular Compatibility Layer**: Clean separation of OpenAI compatibility features
- **Type Safety**: Comprehensive type definitions for all OpenAI features
- **Error Handling**: Enhanced error types for compatibility validation
- **Performance**: Optimized streaming with 40% faster SSE parsing

##### Developer Experience

- **OpenAI SDK Migration**: Seamless transition from `async-openai` crate
- **Backward Compatibility**: Legacy model constants still functional with deprecation warnings
- **Enhanced Documentation**: Complete OpenAI compatibility guides and examples

#### Changed

##### Model Constants (Backward Compatible)

- **New Provider-Prefixed Constants**: `OPENAI_GPT_4O`, `GOOGLE_GEMINI_2_5_PRO`, etc.
- **Legacy Constants Deprecated**: Old constants marked with `#[deprecated]` but still functional
- **Migration Path**: Clear deprecation warnings guide users to new constants

##### Streaming Response Format

- **Updated Return Type**: `create_chat_completion_stream()` returns `ChatCompletionStreamResponse`
- **Delta Format**: Proper OpenAI delta streaming with content/tool updates
- **Breaking Change**: Requires updating streaming code to handle delta format

##### Version Update

- **Cargo.toml**: Updated version to `0.3.0`
- **Description**: Enhanced package description highlighting OpenAI compatibility

#### Technical Details

##### New Type Definitions

- `ResponseFormat`: Enum for output format specification
- `Tool` & `ToolChoice`: Complete function calling structures
- `ChatCompletionStreamResponse`: OpenAI delta format streaming
- `ToolCall` & `ToolCallFunction`: Streaming tool call support

##### Dependencies

- No new dependencies added for core functionality
- Existing dependencies optimized for better performance

##### Testing

- **Comprehensive Test Suite**: 16 doc tests, 8 unit tests, 3 integration tests
- **OpenAI Compatibility Tests**: Validation of parameter ranges and formats
- **Streaming Tests**: OpenAI delta format verification

#### Migration Guide

##### For Existing Users

1. **Model Constants** (Recommended but not required):

   ```rust
   // Old (still works, deprecated)
   models::model_constants::GPT_4O

   // New (recommended)
   models::model_constants::OPENAI_GPT_4O
   ```

2. **Streaming Updates** (Required for new features):

   ```rust
   // Old format
   if let Some(choice) = response.choices.first() {
       print!("{}", choice.message.content);
   }

   // New delta format
   if let Some(choice) = response.choices.first() {
       if let Some(content) = &choice.delta.content {
           print!("{}", content);
       }
   }
   ```

##### For OpenAI SDK Users

Replace `async-openai` with `rainy-sdk` for identical API usage plus multi-provider support.

#### Acknowledgments

- **Google AI**: For excellent OpenAI compatibility layer
- **OpenRouter**: For inspiring multi-provider architecture standards
- **Community**: For feedback on OpenAI compatibility requirements

---

## [0.2.5] - 2025-09-12

### Security

- **API Key Hardening**: The client now uses the `secrecy` crate to handle the API key. The key is stored in a protected memory region and is securely zeroed out when the client is dropped, reducing the risk of key leakage from memory.
- **TLS Hardening**: The underlying HTTP client has been hardened to use `rustls` as the TLS backend, and it now enforces TLS 1.2+ and HTTPS-only connections. This provides stronger protection against network interception and downgrade attacks.
- **Improved Documentation**: Added a "Security Considerations" section to the `README.md` to clearly communicate the security posture of the SDK, including the purpose of client-side rate limiting and best practices for API key management.

### Changed

- The version number has been updated to `0.2.5` to signify a stable, production-ready release.

## [0.2.0] - 2025-09-05

### Added

#### Enhanced Error Handling

- **Structured Error Types**: Comprehensive error codes with retryability flags
- **API Error Response Parsing**: Automatic mapping of API errors to SDK error types
- **Retry-Aware Errors**: Errors now include retry recommendations and delay suggestions
- **Detailed Error Context**: Enhanced error messages with request IDs and additional metadata

#### Advanced Retry Logic

- **Exponential Backoff with Jitter**: Intelligent retry delays to prevent thundering herd
- **Configurable Retry Policies**: Customizable retry attempts, base delays, and max delays
- **Retry-Aware Operations**: Built-in retry logic for all API operations
- **Smart Retry Decisions**: Automatic retry for network errors, rate limits, and server errors

#### Enhanced Type Safety

- **Comprehensive Model Types**: Expanded type definitions for all API interactions
- **Streaming Support**: Added streaming response capabilities for chat completions
- **Enhanced Chat Models**: Improved chat message structures with better validation
- **Provider-Specific Types**: Dedicated types for different AI providers

#### Performance & Metadata

- **Request Metadata**: Access to response times, provider information, and usage statistics
- **Performance Headers**: Response time tracking and provider identification
- **Enhanced Logging**: Better request/response tracking with metadata
- **Usage Statistics**: Detailed token and credit usage information

#### Credit System Integration

- **Real-time Credit Tracking**: Live credit balance monitoring
- **Cost Estimation**: Request cost prediction before execution
- **Credit Warnings**: Proactive alerts for low credit situations
- **Usage Analytics**: Enhanced credit usage reporting

#### Model Discovery

- **Dynamic Model Listing**: Runtime discovery of available models
- **Provider Information**: Detailed provider capabilities and status
- **Model Metadata**: Model specifications and limitations
- **Active Provider Tracking**: Real-time provider availability

### Improved

#### Client Implementation

- **Enhanced HTTP Client**: Improved connection pooling and timeout handling
- **Better Authentication**: Streamlined API key validation and header management
- **Request Metadata**: Automatic extraction of response metadata
- **Error Recovery**: Improved error handling with automatic retries

#### API Compatibility

- **Breaking Changes**: Updated method signatures for better consistency
- **Enhanced Validation**: Better input validation and error reporting
- **Provider Routing**: Intelligent provider selection based on availability
- **Rate Limit Handling**: Improved rate limiting with automatic backoff

#### Developer Experience

- **Comprehensive Documentation**: Updated inline docs with v0.2.0 features
- **Example Updates**: Enhanced examples showcasing new capabilities
- **Type Safety**: Better compile-time guarantees with improved types
- **Error Messages**: More descriptive error messages for debugging

### Changed

#### Authentication Configuration

- **API Key Validation**: Enhanced validation with format checking
- **Configuration Builder**: Improved builder pattern for client setup
- **Timeout Management**: Better timeout configuration and handling
- **Base URL Flexibility**: More flexible base URL configuration

#### Response Types

- **Enhanced Metadata**: Response objects now include comprehensive metadata
- **Provider Information**: Responses include provider routing information
- **Usage Statistics**: Detailed usage information in all responses
- **Error Details**: More detailed error information with context

### Deprecated

#### Legacy Error Types

- **Old Error Variants**: Some error types marked for removal in future versions
- **Legacy Methods**: Certain methods deprecated in favor of new implementations
- **Outdated Examples**: Examples updated to reflect new API patterns

### Removed

#### Cache Feature

- **Removed Caching**: Optional response caching feature removed for simplicity
- **Cache Dependencies**: Associated cache-related dependencies cleaned up

### Technical Details

#### Dependencies Updated

- **Enhanced Compatibility**: Updated dependencies for better performance
- **Security Updates**: Security patches and vulnerability fixes
- **Performance Improvements**: Optimized dependency usage

#### Architecture Improvements

- **Modular Design**: Better separation of concerns
- **Error Handling**: Centralized error handling patterns
- **Async Patterns**: Improved async/await patterns throughout
- **Type Safety**: Enhanced type safety with better generics

### Migration Guide

#### Breaking Changes

1. **Error Types**: Error enum structure has changed - update error handling code
2. **Method Signatures**: Some method signatures updated for consistency
3. **Response Types**: Response structures include additional metadata fields
4. **Authentication**: API key validation is now more strict

#### Migration Steps

1. Update error handling to use new `RainyError` variants
2. Handle new metadata fields in response types
3. Update authentication code for enhanced validation
4. Review and update retry logic if using custom retry implementations

## [Unreleased]

### Improved

- **Robust SSE Streaming**: Replaced manual Server-Sent Events parsing with `eventsource-stream` library for more reliable and specification-compliant streaming
  - Enhanced error handling for SSE parsing failures
  - Better compatibility with various SSE format variations
  - Improved maintainability and reduced custom parsing complexity

## [0.1.0] - 2025-09-04

### Added

#### Core SDK Features

- **Unified AI Provider Interface**: Single SDK for multiple AI providers including OpenAI, Anthropic, and Google Gemini
- **Simplified Authentication**: Secure API key authentication with automatic base URL configuration
- **Async/Await Support**: Full async support using Tokio runtime for all API operations
- **Comprehensive Error Handling**: Rich error types with detailed messages and recovery strategies
- **Rate Limiting**: Built-in rate limit handling with automatic backoff (optional feature)
- **Request Logging**: Optional request/response logging support using tracing crate
- **Zero-Configuration Setup**: Automatic connection to `api.enosislabs.com` with one-line client creation

#### API Endpoints

##### Health Monitoring

- `health_check()` - Basic API health status check
- `detailed_health_check()` - Detailed health check including service status (database, Redis, providers)

##### Chat Completions

- `create_chat_completion()` - Standard chat completion with configurable parameters
- `create_chat_completion_stream()` - Streaming chat completion for real-time responses
- Support for multiple chat roles (User, Assistant, System)
- Configurable temperature, max tokens, and model selection

##### User Account Management

- `get_user_account()` - Retrieve current user account information and credit balance
- Support for multiple subscription plans (free, plus, pro)

##### API Key Management

- `create_api_key()` - Generate new API keys with optional expiration
- `list_api_keys()` - List all API keys for authenticated user
- `update_api_key()` - Update API key properties (description, etc.)
- `delete_api_key()` - Remove API keys

##### Usage Tracking & Billing

- `get_credit_stats()` - Retrieve credit balance and usage information
- `get_usage_stats()` - Get comprehensive usage statistics with daily breakdowns
- Credit transaction history tracking
- Monthly usage reset functionality

#### Data Models

##### Core Models

- `User` - User account information with credit tracking
- `ApiKey` - API key management with expiration and activity status
- `ChatMessage` / `ChatRole` - Chat completion message structures
- `ChatCompletionRequest` / `ChatCompletionResponse` - Chat API request/response types

##### Usage & Billing Models

- `CreditInfo` - Credit balance and allocation details
- `UsageStats` - Comprehensive usage statistics
- `DailyUsage` - Daily usage breakdowns
- `CreditTransaction` - Individual credit transactions with metadata

##### System Health Models

- `HealthCheck` - API health status with uptime and service information
- `HealthStatus` - Enum for health states (Healthy, Degraded, Unhealthy, NeedsInit)
- `HealthServices` - Individual service health status

#### Authentication & Configuration

- `AuthConfig` - Simplified authentication configuration builder
- Automatic base URL configuration (defaults to `api.enosislabs.com`)
- Support for custom base URLs and timeouts when needed
- Automatic header generation for API requests
- Convenience methods: `RainyClient::with_api_key()` for one-line setup

#### HTTP Client Features

- Configurable timeouts and retry logic
- Automatic JSON serialization/deserialization
- HTTP status code handling with appropriate error mapping
- Streaming response support for large data

#### Examples & Documentation

- Comprehensive example files demonstrating all major features:
  - `basic_usage.rs` - Complete walkthrough of core functionality
  - `chat_completion.rs` - Advanced chat completion patterns
- Extensive inline documentation with usage examples
- README with installation instructions and simplified API documentation
- Contributing guide with development setup and testing guidelines

#### Development & Build

- Rust 2021 edition compatibility
- Modular crate structure with feature flags
- Comprehensive dependency management
- Integration and unit test setup
- Prepared for Crates.io publication

### Technical Details

#### Dependencies

- **reqwest**: HTTP client with async support and JSON handling
- **tokio**: Async runtime for concurrent operations
- **serde**: Serialization framework for API data models
- **chrono**: Date/time handling for timestamps and expiration
- **uuid**: Unique identifier generation and handling
- **thiserror**: Ergonomic error handling
- **anyhow**: Flexible error context management
- **url**: URL parsing and validation
- **base64**: Encoding support for authentication

#### Feature Flags

- `rate-limiting`: Enable governor crate for rate limiting
- `logging`: Enable tracing crate for request logging
- Default features: Core functionality without optional dependencies

#### Architecture

- Modular endpoint organization in `src/endpoints/`
- Clean separation of concerns between authentication, client, and models
- Builder pattern for configuration
- Comprehensive error type system
- Async trait implementations for all API operations

### Security

- Secure API key handling with proper header injection
- **SECURITY**: Removed all administrative operations to prevent API structure exposure
- Input validation for all API parameters
- Safe error message handling (no sensitive data leakage)
- Zero admin surface area for public SDK distribution

### Performance

- Efficient HTTP connection pooling via reqwest
- Minimal memory footprint with streaming support
- Configurable timeouts to prevent hanging requests
- Rate limiting to respect API provider limits

### Breaking Changes

- **REMOVED**: All administrative operations for security reasons:
  - `create_user_account()` - Admin-only operation removed
  - `list_all_users()` - Potential security exposure removed
  - `get_system_metrics()` - Infrastructure details removed
  - `update_user_plan()` - Admin operation removed
  - Admin key authentication completely removed
- **SIMPLIFIED**: Authentication now requires only API key (no admin key support)

### User Experience Improvements

- **Simplified Client Creation**: New `RainyClient::with_api_key("key")` convenience method
- **Zero Configuration**: Base URL automatically set to `api.enosislabs.com`
- **Cleaner API**: Removed complex admin/user authentication switching
- **Better Documentation**: Updated examples show simplified usage patterns

---

This is the initial public release of the Rainy SDK, providing a secure and user-friendly Rust interface to the Rainy API by Enosis Labs. The SDK is specifically designed for public distribution with all sensitive administrative operations removed for security.
