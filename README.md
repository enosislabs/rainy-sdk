# 🌧️ Rainy SDK v0.2.1

[![Crates.io](https://img.shields.io/crates/v/rainy-sdk.svg)](https://crates.io/crates/rainy-sdk)
[![Documentation](https://docs.rs/rainy-sdk/badge.svg)](https://docs.rs/rainy-sdk)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/enosislabs/rainy-sdk)

The official Rust SDK for the **Rainy API by Enosis Labs** - a unified interface for multiple AI providers including OpenAI, Anthropic, Google Gemini, and more.

## ✨ Features

- **🚀 Unified API**: Single interface for multiple AI providers
- **🔐 Type-Safe Authentication**: Secure API key management with validation
- **⚡ Async/Await**: Full async support with Tokio runtime
- **📊 Rich Metadata**: Response times, provider info, token usage, credit tracking
- **🛡️ Enhanced Error Handling**: Comprehensive error types with retryability
- **🔄 Intelligent Retry**: Exponential backoff with jitter for resilience
- **📈 Rate Limiting**: Optional governor-based rate limiting
- **📚 Rich Documentation**: Complete API documentation with practical examples

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rainy-sdk = "0.2.1"
tokio = { version = "1.0", features = ["full"] }
```

Or install with cargo:

```bash
cargo add rainy-sdk
```

### Optional Features

Enable additional features as needed:

```toml
[dependencies]
rainy-sdk = { version = "0.2.1", features = ["rate-limiting", "tracing"] }
```

Available features:

- `rate-limiting`: Built-in rate limiting with the `governor` crate.
- `tracing`: Request/response logging with the `tracing` crate.

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
├── client.rs          # Main API client with request handling
├── auth.rs            # Authentication and authorization logic
├── models.rs          # Data structures and serialization
├── error.rs           # Comprehensive error handling
├── retry.rs           # Retry logic with exponential backoff
├── endpoints/         # API endpoint implementations
│   ├── chat.rs        # Chat completion endpoints
│   ├── health.rs      # Health check and monitoring
│   ├── keys.rs        # API key operations
│   ├── usage.rs       # Usage statistics and billing
│   └── user.rs        # User account management
└── lib.rs             # Public API and module exports
```

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
