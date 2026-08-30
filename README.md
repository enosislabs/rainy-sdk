# Rainy SDK

[![Crates.io](https://img.shields.io/crates/v/rainy-sdk.svg)](https://crates.io/crates/rainy-sdk)
[![Documentation](https://docs.rs/rainy-sdk/badge.svg)](https://docs.rs/rainy-sdk)
[![CI](https://github.com/enosislabs/rainy-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/enosislabs/rainy-sdk/actions/workflows/ci.yml)
[![Security](https://github.com/enosislabs/rainy-sdk/actions/workflows/security.yml/badge.svg)](https://github.com/enosislabs/rainy-sdk/actions/workflows/security.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

Rust SDK for the Rainy API. It provides typed, asynchronous clients for OpenAI-compatible Chat and Responses, Anthropic Messages, embeddings, model discovery, web search, health probes, and JWT-backed account operations.

## Installation

```bash
cargo add rainy-sdk@0.6.16
```

Or add it to `Cargo.toml`:

```toml
[dependencies]
rainy-sdk = "0.6.16"
tokio = { version = "1.53", features = ["macros", "rt-multi-thread"] }
```

The crate requires Rust 1.98.0 or newer and uses Tokio for asynchronous operations. HTTPS is handled with Rustls.

Optional features:

| Feature | Purpose |
| --- | --- |
| `rate-limiting` | Built-in request limiting, enabled by default. |
| `tracing` | Optional tracing integration, enabled by default. |
| `legacy` | Compatibility types and older helper methods. |
| `cache` | Compatibility feature for cache integrations. |

## Quick start

Keep your API key in an environment variable and create a `RainyClient`:

```rust,no_run
use rainy_sdk::{ChatCompletionRequest, ChatMessage, RainyClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RainyClient::with_api_key(std::env::var("RAINY_API_KEY")?)?;
    let request = ChatCompletionRequest::new(
        "provider/model-id",
        vec![ChatMessage::user("Explain ownership in Rust")],
    )
    .with_max_tokens(300)
    .with_temperature(0.2);

    let (response, _) = client.chat_completion(request).await?;
    if let Some(choice) = response.choices.first() {
        println!("{}", choice.message.content);
    }
    Ok(())
}
```

## Which client should I use?

| Need | Client |
| --- | --- |
| Chat, Responses, Anthropic Messages, embeddings, search, health/readiness, and model discovery | `RainyClient` |
| Login, registration, profiles, usage, billing, organization, and API-key management | `RainySessionClient` |

`RainyClient` uses an API key. `RainySessionClient` uses a user session. Keeping these clients separate makes it clear which credential a feature requires.

The session client intentionally does not send refresh tokens to `/auth/refresh`: the current API returns `REFRESH_UNSUPPORTED` because Better Auth manages refresh behavior separately. Re-authenticate with `login` or `register` when an access session is no longer valid.

## Configuration

Use `AuthConfig` when you need a custom timeout, retry policy, user agent, or compatible service URL:

```rust,no_run
use rainy_sdk::{AuthConfig, RainyClient};

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let config = AuthConfig::new(std::env::var("RAINY_API_KEY")?)
    .with_base_url("https://api.example.com")
    .with_timeout(60)
    .with_max_retries(3);
let client = RainyClient::with_config(config)?;
# let _ = client;
# Ok(())
# }
```

## Model discovery

Model availability can change independently of the SDK. Use the catalog instead of maintaining a hardcoded list:

```rust,no_run
use rainy_sdk::{ModelSelectionCriteria, ModelTier, RainyClient};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let models = client
    .select_models(ModelSelectionCriteria {
        allowed_tiers: vec![ModelTier::Core, ModelTier::Standard],
        require_tools: Some(true),
        ..Default::default()
    })
    .await?;

for model in models {
    println!("{}", model.id);
}
# Ok(())
# }
```

Use `get_models_catalog` when you need the complete typed catalog response.

## Chat completions

`ChatCompletionRequest` covers the common chat flow. For full OpenAI-compatible message history, tool calls, tool results, and multimodal content, use `OpenAIChatCompletionRequest`.

```rust,no_run
use rainy_sdk::{ChatCompletionRequest, ChatMessage, RainyClient};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let request = ChatCompletionRequest::new(
    "provider/model-id",
    vec![
        ChatMessage::system("You are a concise assistant."),
        ChatMessage::user("Summarize this paragraph."),
    ],
)
.with_max_completion_tokens(500)
.with_temperature(0.2);

let (response, metadata) = client.chat_completion(request).await?;
println!("{}", response.choices[0].message.content);
println!("request id: {:?}", metadata.request_id);
# Ok(())
# }
```

## Streaming

Streaming methods return asynchronous streams of response chunks. Add `futures = "0.3"` to your application to use `StreamExt`:

```rust,no_run
use futures::StreamExt;
use rainy_sdk::{ChatCompletionRequest, ChatMessage, RainyClient};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let request = ChatCompletionRequest::new(
    "provider/model-id",
    vec![ChatMessage::user("Write a short haiku about Rust")],
);
let mut stream = client.chat_completion_stream(request).await?;

while let Some(chunk) = stream.next().await {
    if let Some(choice) = chunk?.choices.first()
        && let Some(content) = &choice.delta.content
    {
        print!("{content}");
    }
}
# Ok(())
# }
```

The SDK also exposes typed stream-event methods for applications that need more than standard response chunks.

## Reasoning and capabilities

Reasoning controls are typed and preserve the wire form accepted by Rainy. Use an effort control, an adaptive control, or a numeric budget only when the selected model's catalog metadata declares it:

```rust,no_run
use rainy_sdk::{ChatCompletionRequest, ChatMessage, RainyClient, ReasoningEffort};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let request = ChatCompletionRequest::new(
    "provider/model-id",
    vec![ChatMessage::user("Compare these two designs")],
)
.with_reasoning_effort(ReasoningEffort::High)
.with_include_reasoning(true);

let (response, _) = client.chat_completion(request).await?;
println!("{}", response.choices[0].message.content);
# Ok(())
# }
```

`RainyClient::get_models_catalog` returns `rainy_capabilities_v2`, including accepted parameters, provider profiles, effort values, and numeric budget sentinels. `RainyClient::build_reasoning_config` uses that metadata to construct an exact nested request and returns no configuration when the capability is not declared. The complete API-to-SDK route classification is available as the machine-readable [`docs/API_CAPABILITY_MATRIX.json`](docs/API_CAPABILITY_MATRIX.json).

## Anthropic Messages API

Use the typed Messages surface when an Anthropic-compatible request or native Messages stream is required. It sends `anthropic-version: 2023-06-01` by default and preserves native event names such as `message_start`, `content_block_delta`, and `message_stop`:

```rust,no_run
use rainy_sdk::{AnthropicMessage, AnthropicMessageRequest, RainyClient};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let request = AnthropicMessageRequest::new(
    "provider/model-id",
    vec![AnthropicMessage::user("Explain Rust lifetimes")],
    600,
);

let response = client.create_message(request).await?;
println!("{}", response.text());
# Ok(())
# }
```

## Responses API

`ResponsesRequest` supports text and structured input, reasoning controls, hosted tools, function tools, metadata, streaming, and multi-turn continuation:

```rust,no_run
use rainy_sdk::{RainyClient, ResponsesRequest};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let request = ResponsesRequest::text("provider/model-id", "Summarize this document")
    .with_reasoning_effort("medium")
    .with_max_output_tokens(800)
    .add_web_search_tool();

let (response, _) = client.create_response(request).await?;
println!("{}", response.text().unwrap_or_default());
# Ok(())
# }
```

For custom function workflows, use `ResponsesRequest::function_call_output` and `with_previous_response_id` to continue a response.

The current API does not expose the former `max_tool_calls`, prompt-cache-options, or standalone Agents contract. Use the Responses `tools`/continuation fields and the explicit capability metadata instead; the SDK preserves unknown fields only through documented extension maps.

## Embeddings

The embeddings API accepts text or token inputs and can return floating-point or base64 values:

```rust,no_run
use rainy_sdk::{EmbeddingEncodingFormat, EmbeddingsRequest, RainyClient};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let mut request = EmbeddingsRequest::text("provider/embedding-model-id", "Rainy embeddings");
request.encoding_format = Some(EmbeddingEncodingFormat::Float);
request.dimensions = Some(768);

let response = client.create_embeddings(request).await?;
println!("{} embedding(s)", response.data.len());
# Ok(())
# }
```

## Search

Use `search` for typed web results and `search_extract` for content from a list of URLs. `research` remains available for compatible research workflows.

```rust,no_run
use rainy_sdk::{RainyClient, ResearchDepth};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let results = client
    .search("Rust release notes", Some(ResearchDepth::Basic), Some(10))
    .await?;

for result in results.results {
    println!("{:?}: {:?}", result.title, result.url);
}
# Ok(())
# }
```

## User sessions

Use `RainySessionClient` for user and account workflows. Login keeps the access token in memory for subsequent calls:

```rust,no_run
use rainy_sdk::RainySessionClient;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let mut session = RainySessionClient::new()?;
session
    .login(
        &std::env::var("RAINY_EMAIL")?,
        &std::env::var("RAINY_PASSWORD")?,
    )
    .await?;

let profile = session.me().await?;
println!("{}", profile.email);
# Ok(())
# }
```

The API's `/auth/refresh` route currently returns `REFRESH_UNSUPPORTED`; the SDK reports that as `RainyError::FeatureNotAvailable` and does not transmit the supplied refresh token. Keep session tokens private and do not log or commit credentials.

## Errors and retries

Fallible methods return `Result<T, RainyError>`. The error provides a stable SDK type, a machine-readable code where available, and helper methods such as `is_retryable` and `retry_after`:

```rust,no_run
use rainy_sdk::{RainyClient, RainyError};

# async fn example(client: &RainyClient, request: rainy_sdk::ChatCompletionRequest) {
match client.chat_completion(request).await {
    Ok((response, _)) => println!("{}", response.choices[0].message.content),
    Err(RainyError::AccessDenied { message, .. }) => eprintln!("access denied: {message}"),
    Err(error) if error.is_retryable() => eprintln!("retryable error: {error}"),
    Err(error) => eprintln!("request failed: {error}"),
}
# }
```

## Development

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the contributor workflow, [`CHANGELOG.md`](CHANGELOG.md) for release history, and [`SECURITY.md`](SECURITY.md) for security reports.

```bash
cargo fmt --all -- --check
cargo check --locked --all-targets --all-features
cargo test --locked --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo doc --locked --all-features --no-deps
```

## License

Apache-2.0. See [`LICENSE`](LICENSE).
