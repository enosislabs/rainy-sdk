# 🌧️ Rainy SDK v0.2.0

[![Crates.io](https://img.shields.io/crates/v/rainy-sdk.svg)](https://crates.io/crates/rainy-sdk)
[![Documentation](https://docs.rs/rainy-sdk/badge.svg)](https://docs.rs/rainy-sdk)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

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
rainy-sdk = { version = "0.2.0", features = ["rate-limiting", "tracing"] }
```

Available features:

- `rate-limiting`: Built-in rate limiting with governor crate
- `tracing`: Request/response logging with tracing crate

## 🚀 Quick Start

```rust
use rainy_sdk::{RainyClient, ChatCompletionRequest, ChatMessage, models};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize client with your API key
    let client = RainyClient::with_api_key("ra-your-api-key")?;

    // Simple chat completion
    let response = client.simple_chat(models::model_constants::GPT_4O, "Hello! Tell me a joke.").await?;
    println!("Response: {}", response);

    // Advanced usage with metadata
    let request = ChatCompletionRequest::new(
        models::model_constants::CLAUDE_SONNET_4,
        vec![ChatMessage::user("Explain quantum computing")]
    )
    .with_temperature(0.7)
    .with_max_tokens(200);

    let (response, metadata) = client.chat_completion(request).await?;
    println!("Response: {}", response.choices[0].message.content);
    println!("Provider: {:?}", metadata.provider);
    println!("Response time: {}ms", metadata.response_time.unwrap_or_default());

    Ok(())
}
```

## 📖 API Documentation

### Authentication

The SDK uses API key authentication:

#### API Key Authentication

```rust
use rainy_sdk::RainyClient;

// Simplest possible setup
let client = RainyClient::with_api_key("ra-20250125143052Axxxxxxxxxxxxxxxxxxxx")?;
```

### Core Operations

#### Health Check

```rust
let health = client.health_check().await?;
println!("API Status: {:?}", health.status);
```

#### User Account Management

```rust
// Get user account information
let user = client.get_user_account().await?;
println!("Credits: {:.2}", user.current_credits);

// Get credit information
let credits = client.get_credit_info().await?;
println!("Balance: {:.2}", credits.current_balance);
```

#### Chat Completions

```rust
let messages = vec![
    ChatMessage {
        role: ChatRole::System,
        content: "You are a helpful assistant.".to_string(),
    },
    ChatMessage {
        role: ChatRole::User,
        content: "Explain quantum computing in simple terms.".to_string(),
    }
];

let request = ChatCompletionRequest {
    model: "gpt-4".to_string(),
    messages,
    max_tokens: Some(500),
    temperature: Some(0.7),
    stream: None,
};

let response = client.create_chat_completion(request).await?;
for choice in response.choices {
    println!("Response: {}", choice.message.content);
}
```

#### Usage Statistics

```rust
// Get usage stats for the last 30 days
let usage = client.get_usage_stats(Some(30)).await?;
println!("Total requests: {}", usage.total_requests);
println!("Total tokens: {}", usage.total_tokens);

// Show daily usage breakdown
for daily in usage.daily_usage {
    println!("{}: {:.2} credits used", daily.date, daily.credits_used);
}
```

#### API Key Management

```rust
// List all API keys
let keys = client.list_api_keys().await?;
for key in keys {
    println!("Key: {}... Active: {}", &key.key[..20], key.is_active);
}

// Create new API key
let new_key = client.create_api_key(Some("My new key".to_string())).await?;
println!("Created key: {}", new_key.key);

// Delete API key
client.delete_api_key(new_key.id).await?;
```

## 🧪 Examples

Explore the `examples/` directory for comprehensive usage examples:

- **Basic Usage** (`examples/basic_usage.rs`): Complete walkthrough of all SDK features
- **Chat Completion** (`examples/chat_completion.rs`): Advanced chat completion patterns

Run examples with:

```bash
# Set your API key
export RAINY_API_KEY="your-api-key-here"

# Run basic usage example
cargo run --example basic_usage

# Run chat completion example
cargo run --example chat_completion
```

## 🏗️ Architecture

The SDK is built with a modular architecture:

```
src/
├── client.rs          # Main API client with request handling
├── auth.rs            # Authentication and authorization logic
├── models.rs          # Data structures and serialization
├── error.rs           # Comprehensive error handling
├── endpoints/         # API endpoint implementations
│   ├── user.rs        # User account management
│   ├── keys.rs        # API key operations
│   ├── chat.rs        # Chat completion endpoints
│   ├── usage.rs       # Usage statistics and billing
│   └── health.rs      # Health check and monitoring
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
