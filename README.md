# 🌧️ Rainy SDK v0.2.0

[![Crates.io](https://img.shields.io/crates/v/rainy-sdk.svg)](https://crates.io/crates/rainy-sdk)
[![Documentation](https://docs.rs/rainy-sdk/badge.svg)](https://docs.rs/rainy-sdk)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

The official Rust SDK for the **Rainy API by Enosis Labs** - a unified interface for multiple AI providers including OpenAI, Anthropic, Google Gemini, and more.

## ✨ Features

- **🚀 Unified API**: Single interface for multiple AI providers.
- **🔐 Type-Safe Authentication**: Secure API key management with validation.
- **⚡ Async/Await**: Full async support with Tokio runtime.
- **📊 Rich Metadata**: Get response times, provider info, token usage, and credit tracking.
- **🛡️ Enhanced Error Handling**: Comprehensive error types with retryability.
- **🔄 Intelligent Retry**: Exponential backoff with jitter for resilience.
- **📈 Rate Limiting**: Optional governor-based rate limiting.
- **📚 Rich Documentation**: Complete API documentation with practical examples.

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rainy-sdk = "0.2.0"
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

- `rate-limiting`: Built-in rate limiting with the `governor` crate.
- `tracing`: Request/response logging with the `tracing` crate.

## 🚀 Quick Start

```rust
use rainy_sdk::{RainyClient, models::{self, ChatCompletionRequest, ChatMessage}};
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // It's recommended to set your API key as an environment variable.
    env::set_var("RAINY_API_KEY", "ra-your-api-key");

    // Initialize the client. It will automatically read the RAINY_API_KEY.
    let client = RainyClient::new()?;

    // Perform a health check to ensure connectivity.
    let health = client.health_check().await?;
    println!("API Status: {}", health.status);

    // Simple chat completion.
    let response = client.simple_chat(models::model_constants::GPT_4O, "Hello! Tell me a joke.").await?;
    println!("Response: {}", response);

    // Advanced usage with metadata.
    let request = ChatCompletionRequest::new(
        models::model_constants::CLAUDE_SONNET_4,
        vec![ChatMessage::user("Explain quantum computing in simple terms.")]
    )
    .with_temperature(0.7)
    .with_max_tokens(200);

    let (response, metadata) = client.chat_completion(request).await?;
    println!("\nFull Response:\n{:?}", response.choices[0].message.content);
    println!("\nMetadata:\nProvider: {:?}, Response Time: {}ms", metadata.provider, metadata.response_time.unwrap_or_default());

    Ok(())
}
```

## 📖 API Documentation

### Authentication

The SDK can be authenticated by providing an API key directly or by setting the `RAINY_API_KEY` environment variable.

```rust
use rainy_sdk::RainyClient;
use std::env;

# fn main() -> Result<(), rainy_sdk::RainyError> {
// Recommended: Initialize from environment variable
env::set_var("RAINY_API_KEY", "ra-your-api-key");
let client_from_env = RainyClient::new()?;

// Or, provide the key directly
let client_from_key = RainyClient::with_api_key("ra-your-api-key")?;
# Ok(())
# }
```

### Core Operations

#### Health Check

```rust
# async fn example(client: &rainy_sdk::RainyClient) -> rainy_sdk::Result<()> {
let health = client.health_check().await?;
println!("API Status: {}", health.status);
# Ok(())
# }
```

#### User Account Management

```rust
# async fn example(client: &rainy_sdk::RainyClient) -> rainy_sdk::Result<()> {
// Get user account information
let user = client.get_user_account().await?;
println!("User Credits: {:.2}", user.current_credits);

// Get credit balance information
let credits = client.get_credit_info().await?;
println!("Current Balance: {:.2}", credits.current_credits);
# Ok(())
# }
```

#### Chat Completions

```rust
# use rainy_sdk::{models::{ChatMessage, ChatCompletionRequest}};
# async fn example(client: &rainy_sdk::RainyClient) -> rainy_sdk::Result<()> {
let messages = vec![
    ChatMessage::system("You are a helpful assistant that provides concise answers."),
    ChatMessage::user("Explain quantum computing in one sentence."),
];

let request = ChatCompletionRequest::new("gpt-4o", messages)
    .with_max_tokens(100)
    .with_temperature(0.5);

let response = client.create_chat_completion(request).await?;
if let Some(choice) = response.choices.first() {
    println!("Response: {}", choice.message.content);
}
# Ok(())
# }
```

#### Usage Statistics

```rust
# async fn example(client: &rainy_sdk::RainyClient) -> rainy_sdk::Result<()> {
// Get usage stats for the last 30 days
let usage = client.get_usage_stats(Some(30)).await?;
println!("Total requests: {}", usage.total_requests);
println!("Total tokens: {}", usage.total_tokens);

// Show daily usage breakdown
for daily in usage.daily_usage {
    println!("{}: {:.2} credits used", daily.date, daily.credits_used);
}
# Ok(())
# }
```

#### API Key Management

```rust
# async fn example(client: &rainy_sdk::RainyClient) -> rainy_sdk::Result<()> {
// List all API keys
let keys = client.list_api_keys().await?;
for key in &keys {
    println!("Key ID: {}, Description: {}", key.id, key.description.as_deref().unwrap_or("N/A"));
}

// Create a new API key (requires master key privileges)
let new_key = client.create_api_key("My new temporary key", Some(30)).await?;
println!("Created key: {}", new_key.key); // The key is only returned on creation

// Delete the API key
client.delete_api_key(&new_key.id.to_string()).await?;
println!("API key deleted successfully.");
# Ok(())
# }
```

## 🧪 Examples

Explore the `examples/` directory for comprehensive usage examples:

- **Basic Usage** (`examples/basic_usage.rs`): Complete walkthrough of all SDK features.
- **Chat Completion** (`examples/chat_completion.rs`): Advanced chat completion patterns.

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
