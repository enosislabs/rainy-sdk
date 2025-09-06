# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

---

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
