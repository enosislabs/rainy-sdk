# 🌧️ Rainy SDK v0.5.1

[![Crates.io](https://img.shields.io/crates/v/rainy-sdk.svg)](https://crates.io/crates/rainy-sdk)
[![Documentation](https://docs.rs/rainy-sdk/badge.svg)](https://docs.rs/rainy-sdk)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/enosislabs/rainy-sdk)
[![Rust Version](https://img.shields.io/badge/rust-1.92.0%2B-orange.svg)](https://www.rust-lang.org/)

The official Rust SDK for the **Rainy API by Enosis Labs** - a unified interface for multiple AI providers including OpenAI, Google Gemini, Groq, Cerebras, and Enosis Labs' own Astronomer models. Features advanced thinking capabilities, multimodal support, thought signatures, and full OpenAI compatibility.

## ✨ Features

- **🎯 Full OpenAI Compatibility**: Drop-in replacement for OpenAI SDK with enhanced features
- **🚀 Unified Multi-Provider API**: Single interface for OpenAI, Google Gemini, Groq, Cerebras, and Enosis Labs Astronomer models
- **🧠 Advanced Thinking Capabilities**: Gemini 3 and 2.5 series models with configurable reasoning levels and thought signatures
- **🔐 Type-Safe Authentication**: Secure API key management with the `secrecy` crate
- **⚡ Async/Await**: Full async support with Tokio runtime
- **📊 Rich Metadata**: Response times, provider info, token usage, credit tracking, and thinking token counts
- **🛡️ Enhanced Error Handling**: Comprehensive error types with retryability and detailed diagnostics
- **🔄 Intelligent Retry**: Exponential backoff with jitter for resilience
- **📈 Rate Limiting**: Optional governor-based rate limiting
- **🔧 Advanced Parameters**: Support for response_format, tools, tool_choice, reasoning_effort, logprobs, and streaming
- **🌐 Web Search Integration**: Built-in Tavily-powered web search with content extraction
- **👥 Cowork Integration**: Tier-based feature gating with Free/GoPlus/Plus/Pro/ProPlus plans
- **🎨 Multimodal Support**: Image processing and multimodal capabilities (coming soon)
- **📚 Rich Documentation**: Complete API documentation with practical examples

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rainy-sdk = "0.5.1"
tokio = { version = "1.47", features = ["full"] }
```

Or installation with cargo:

```bash
cargo add rainy-sdk
```

### Requirements

- **Rust**: 1.92.0 or later
- **Platform Support**: macOS, Linux, Windows

### Optional Features

Enable additional features as needed:

```toml
[dependencies]
rainy-sdk = { version = "0.5.1", features = ["rate-limiting", "tracing", "cowork"] }
```

Available features:

- `rate-limiting`: Built-in rate limiting with the `governor` crate
- `tracing`: Request/response logging with the `tracing` crate
- `cowork`: Cowork integration for tier-based feature gating (enabled by default)

## 🎯 OpenAI Compatibility

Rainy SDK v0.5.1 provides **100% OpenAI API compatibility** while extending support to additional providers. Use Rainy SDK as a drop-in replacement for the official OpenAI SDK:

```rust
use rainy_sdk::{models, ChatCompletionRequest, ChatMessage, RainyClient};

// Works exactly like OpenAI SDK
let client = RainyClient::with_api_key("your-rainy-api-key")?;

let request = ChatCompletionRequest::new(
    models::model_constants::OPENAI_GPT_4O, // or GOOGLE_GEMINI_2_5_PRO
    vec![ChatMessage::user("Hello!")]
)
.with_temperature(0.7)
.with_response_format(models::ResponseFormat::JsonObject);

let (response, metadata) = client.chat_completion(request).await?;
```

### Supported Models (100% OpenAI Compatible)

| Provider | Models | Features |
|----------|--------|----------|
| **OpenAI** | `gpt-4o`, `gpt-5`, `gpt-5-pro`, `o3`, `o4-mini` | ✅ Native OpenAI API |
| **Google Gemini 3** | `gemini-3-pro-preview`, `gemini-3-flash-preview`, `gemini-3-pro-image-preview` | ✅ Thinking, Thought Signatures, Multimodal |
| **Google Gemini 2.5** | `gemini-2.5-pro`, `gemini-2.5-flash`, `gemini-2.5-flash-lite` | ✅ Thinking, Dynamic Reasoning |
| **Groq** | `llama-3.1-8b-instant`, `llama-3.3-70b-versatile` | ✅ OpenAI-compatible API |
| **Cerebras** | `llama3.1-8b` | ✅ OpenAI-compatible API |
| **Enosis Labs** | `astronomer-1`, `astronomer-1-max`, `astronomer-1.5`, `astronomer-2`, `astronomer-2-pro` | ✅ Native Rainy API |

### Advanced OpenAI Features

- **Tool Calling**: Function calling with `tools` and `tool_choice`
- **Structured Output**: JSON Schema enforcement with `response_format`
- **Reasoning Control**: `reasoning_effort` parameter for Gemini models
- **Log Probabilities**: `logprobs` and `top_logprobs` support
- **Streaming**: OpenAI-compatible delta format streaming with tool calls

## 🧠 Advanced Thinking Capabilities

Rainy SDK supports advanced thinking capabilities for Google Gemini 3 and 2.5 series models, enabling deeper reasoning and thought preservation across conversations.

### Gemini 3 Thinking Features

```rust
use rainy_sdk::{models, ChatCompletionRequest, ChatMessage, RainyClient, ThinkingConfig};

let request = ChatCompletionRequest::new(
    models::model_constants::GOOGLE_GEMINI_3_PRO,
    vec![ChatMessage::user("Solve this complex optimization problem step by step.")]
)
.with_thinking_config(ThinkingConfig::gemini_3(
    models::ThinkingLevel::High, // High reasoning for complex tasks
    true // Include thought summaries in response
));

let (response, metadata) = client.chat_completion(request).await?;
println!("Response: {}", response.choices[0].message.content);
// Access thinking token usage
if let Some(thinking_tokens) = metadata.thoughts_token_count {
    println!("Thinking tokens used: {}", thinking_tokens);
}
```

### Thought Signatures

Preserve reasoning context across conversation turns with encrypted thought signatures:

```rust
use rainy_sdk::{models::*, ChatMessage, EnhancedChatMessage};

let mut conversation = vec![
// Previous messages with thought signatures...
];

// New message with preserved reasoning context
let enhanced_message = EnhancedChatMessage::with_parts(
    MessageRole::User,
    vec![
        ContentPart::text("Now apply this reasoning to the next problem..."),
        // Include thought signature from previous response
        ContentPart::with_thought_signature("encrypted_signature_here".to_string())
    ]
);
```

### Gemini 2.5 Dynamic Thinking

```rust
let config = ThinkingConfig::gemini_2_5(
    -1, // Dynamic thinking budget
    true // Include thoughts
);

let request = ChatCompletionRequest::new(
    models::model_constants::GOOGLE_GEMINI_2_5_PRO,
    messages
)
.with_thinking_config(config);
```

## 🌐 Web Search Integration

Built-in web search powered by Tavily for real-time information retrieval:

```rust
use rainy_sdk::search::{SearchOptions, SearchResponse};

let search_options = SearchOptions {
    query: "latest developments in Rust programming".to_string(),
    max_results: Some(10),
    ..Default::default()
};

let search_results = client.search_web(search_options).await?;
for result in search_results.results {
    println!("{}: {}", result.title, result.url);
}

// Extract content from specific URLs
let extracted = client.extract_content(vec!["https://example.com/article".to_string()]).await?;
println!("Content: {}", extracted.content);
```

## 👥 Cowork Integration

Tier-based feature gating with Free/GoPlus/Plus/Pro/ProPlus plans:

```rust
use rainy_sdk::{CoworkStatus, CoworkClient};

let cowork_client = CoworkClient::new(client);
let status = cowork_client.get_cowork_status().await?;

println!("Plan: {:?}", status.plan);
println!("Remaining uses: {}", status.usage.remaining_uses);

// Check feature availability
if status.can_use_web_research() {
    // Enable web search features
}
if status.can_use_document_export() {
    // Enable document generation
}
```

## 🚀 Quick Start

```rust
use rainy_sdk::{models, ChatCompletionRequest, ChatMessage, RainyClient};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize client with your API key from environment variables
    let api_key = std::env::var("RAINY_API_KEY").expect("RAINY_API_KEY not set");
    let client = RainyClient::with_api_key(api_key)?;

    // Simple chat completion
    let response = client
        .simple_chat(
            models::model_constants::GPT_4O,
            "Hello! Tell me a short story.",
        )
        .await?;
    println!("Simple response: {}", response);

    // Advanced usage with metadata
    let request = ChatCompletionRequest::new(
        models::model_constants::CLAUDE_SONNET_4,
        vec![ChatMessage::user("Explain quantum computing in one sentence")],
    )
    .with_temperature(0.7)
    .with_max_tokens(100);

    let (response, metadata) = client.chat_completion(request).await?;
    println!("\nAdvanced response: {}", response.choices[0].message.content);
    println!("Provider: {:?}", metadata.provider.unwrap_or_default());
    println!("Response time: {}ms", metadata.response_time.unwrap_or_default());

    Ok(())
}
```

## 📖 API Documentation

### Authentication

The SDK uses API key authentication. It's recommended to load the key from an environment variable.

```rust
use rainy_sdk::RainyClient;

// Load API key from environment and create client
let api_key = std::env::var("RAINY_API_KEY").expect("RAINY_API_KEY not set");
let client = RainyClient::with_api_key(api_key)?;
```

### Core Operations

#### Health Check

Verify the API status.

```rust,no_run
# use rainy_sdk::RainyClient;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = RainyClient::with_api_key("dummy")?;
let health = client.health_check().await?;
println!("API Status: {}", health.status);
# Ok(())
# }
```

#### Chat Completions

Create a standard chat completion.

```rust,no_run
# use rainy_sdk::{RainyClient, ChatCompletionRequest, ChatMessage, models};
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = RainyClient::with_api_key("dummy")?;
let messages = vec![
    ChatMessage::system("You are a helpful assistant."),
    ChatMessage::user("Explain quantum computing in simple terms."),
];

let request = ChatCompletionRequest::new(models::model_constants::GPT_4O, messages)
    .with_max_tokens(500)
    .with_temperature(0.7);

let (response, metadata) = client.chat_completion(request).await?;
if let Some(choice) = response.choices.first() {
    println!("Response: {}", choice.message.content);
}
# Ok(())
# }
```

#### Streaming Chat Completions

Receive the response as a stream of events.

```rust,no_run
# use rainy_sdk::{RainyClient, ChatCompletionRequest, ChatMessage, models};
# use futures::StreamExt;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = RainyClient::with_api_key("dummy")?;
let request = ChatCompletionRequest::new(
    models::model_constants::LLAMA_3_1_8B_INSTANT,
    vec![ChatMessage::user("Write a haiku about Rust programming")],
)
.with_stream(true);

let mut stream = client.create_chat_completion_stream(request).await?;

while let Some(chunk) = stream.next().await {
    match chunk {
        Ok(response) => {
            if let Some(choice) = response.choices.first() {
                print!("{}", choice.message.content);
            }
        }
        Err(e) => eprintln!("\nError in stream: {}", e),
    }
}
# Ok(())
# }
```

#### Usage Statistics

Get credit and usage statistics.

```rust,no_run
# use rainy_sdk::RainyClient;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = RainyClient::with_api_key("dummy")?;
// Get credit stats
let credits = client.get_credit_stats(None).await?;
println!("Current credits: {}", credits.current_credits);

// Get usage stats for the last 7 days
let usage = client.get_usage_stats(Some(7)).await?;
println!("Total requests (last 7 days): {}", usage.total_requests);
# Ok(())
# }
```

#### API Key Management

Manage API keys programmatically.

```rust,no_run
# use rainy_sdk::RainyClient;
# async fn example() -> Result<(), Box<dyn std::error::Error>> {
# let client = RainyClient::with_api_key("dummy")?;
// List all API keys
let keys = client.list_api_keys().await?;
for key in keys {
    println!("Key ID: {} - Active: {}", key.id, key.is_active);
}

// Create a new API key
let new_key = client.create_api_key("My new key", Some(30)).await?;
println!("Created key: {}", new_key.key);

// Delete the API key
client.delete_api_key(&new_key.id.to_string()).await?;
# Ok(())
# }
```

## 🧪 Examples

Explore the `examples/` directory for comprehensive usage examples:

- **Basic Usage** (`examples/basic_usage.rs`): Complete walkthrough of all SDK features.
- **Chat Completion** (`examples/chat_completion.rs`): Advanced chat completion patterns.
- **Error Handling** (`examples/error_handling.rs`): Demonstrates how to handle different error types.

Run examples with:

```bash
# Set your API key
export RAINY_API_KEY="your-api-key-here"

# Run basic usage example
cargo run --example basic_usage

# Run chat completion example
cargo run --example chat_completion
```

## 🛡️ Security Considerations

- **API Key Management**: This SDK utilizes the `secrecy` crate to handle the API key, ensuring it is securely stored in memory and zeroed out upon being dropped. However, it is still crucial to manage the `RainyClient`'s lifecycle carefully within your application to minimize exposure.

- **Rate Limiting**: The optional `rate-limiting` feature is intended as a client-side safeguard to prevent accidental overuse and to act as a "good citizen" towards the API. It **is not a security mechanism** and can be bypassed by a malicious actor. For robust abuse prevention, you **must** implement server-side monitoring, usage quotas, and API key management through your Enosis Labs dashboard.

- **TLS Configuration**: The client is hardened to use modern, secure TLS settings (TLS 1.2+ via the `rustls` backend) and to only allow HTTPS connections, providing strong protection against network interception.

## 🏗️ Architecture

The SDK is built with a modular architecture:

```
src/
├── auth.rs            # Authentication and API key management
├── client.rs          # Main API client with request handling
├── cowork.rs          # Tier-based feature gating and capabilities
├── endpoints/         # API endpoint implementations (internal)
├── error.rs           # Comprehensive error handling
├── models.rs          # Data structures and type definitions
├── retry.rs           # Retry logic with exponential backoff
├── search.rs          # Web search and content extraction
└── lib.rs             # Public API and module exports
```

### Key Modules

- **`client.rs`**: Core `RainyClient` with async HTTP handling and response processing
- **`models.rs`**: Complete type system including `ChatCompletionRequest`, `ThinkingConfig`, `EnhancedChatMessage`
- **`auth.rs`**: Secure authentication with the `secrecy` crate for API key management
- **`cowork.rs`**: Integration with Enosis Labs' tier system (Free/GoPlus/Plus/Pro/ProPlus)
- **`search.rs`**: Tavily-powered web search with content extraction capabilities
- **`endpoints/`**: Internal API endpoint implementations (chat, health, keys, usage, user)

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details on:

- Setting up your development environment
- Code style and standards
- Testing guidelines
- Submitting pull requests

## 📄 License

This project is licensed under the Apache License 2.0 - see the [LICENSE](LICENSE) file for details.

## 📞 Contact & Support

- **Website**: [enosislabs.com](https://enosislabs.com)
- **Email**: <hello@enosislabs.com>
- **GitHub**: [github.com/enosislabs](https://github.com/enosislabs)
- **Documentation**: [docs.rs/rainy-sdk](https://docs.rs/rainy-sdk)

## ⚠️ Disclaimer

This SDK is developed by Enosis Labs and is not officially affiliated with any AI provider mentioned (OpenAI, Anthropic, Google, etc.). The Rainy API serves as an independent gateway service that provides unified access to multiple AI providers.

---

<p align="center">
  Made with ❤️ by <a href="https://enosislabs.com">Enosis Labs</a>
</p>
