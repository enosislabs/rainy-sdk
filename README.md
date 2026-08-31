# Rainy SDK

[![Crates.io](https://img.shields.io/crates/v/rainy-sdk.svg)](https://crates.io/crates/rainy-sdk)
[![Documentation](https://docs.rs/rainy-sdk/badge.svg)](https://docs.rs/rainy-sdk)
[![CI](https://github.com/enosislabs/rainy-sdk/actions/workflows/ci.yml/badge.svg)](https://github.com/enosislabs/rainy-sdk/actions/workflows/ci.yml)
[![Security](https://github.com/enosislabs/rainy-sdk/actions/workflows/security.yml/badge.svg)](https://github.com/enosislabs/rainy-sdk/actions/workflows/security.yml)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

`rainy-sdk` is an asynchronous Rust client for protocol-compatible inference
services. It keeps the wire protocol at the center of the API:

- OpenAI-compatible Chat Completions
- OpenAI-compatible Responses
- native Anthropic-compatible Messages
- OpenAI-compatible embeddings
- optional Rainy model discovery, health, search, and response-envelope helpers

The same `RainyClient` can speak to the default Rainy service or to a compatible
custom endpoint. It sends the request you build; it does not perform model
discovery or account calls before inference.

## Installation

```toml
[dependencies]
rainy-sdk = "0.6.50"
tokio = { version = "1.53", features = ["macros", "rt-multi-thread"] }
futures = "0.3"
```

The crate requires Rust 1.98.0 or newer and uses Rustls for HTTPS.

Optional features are additive and disabled by default:

| Feature | Purpose |
| --- | --- |
| `rate-limiting` | Opt-in local request limiting using `governor`. |
| `tracing` | Opt-in retry diagnostics through `tracing`. |
| `cache` | Compatibility feature for future cache integrations. |
| `rainy-account` | Opt-in JWT/session, profile, usage, and API-key management. |
| `legacy` | Opt-in compatibility types and older helper methods. |

For example, enable account/session APIs only in an application that needs
them:

```toml
rainy-sdk = { version = "0.6.50", features = ["rainy-account"] }
```

## Quick start

Keep the API key in an environment variable and create a client:

```rust,no_run
use rainy_sdk::{ChatCompletionRequest, ChatMessage, RainyClient};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = RainyClient::with_api_key(std::env::var("RAINY_API_KEY")?)?;
    let request = ChatCompletionRequest::new(
        "compatible/chat-model",
        vec![ChatMessage::user("Explain ownership in Rust in two sentences.")],
    )
    .with_max_completion_tokens(200)
    .with_temperature(0.2);

    let response = client.create_chat_completion(request).await?;
    println!("{}", response.choices[0].message.content);
    Ok(())
}
```

## Configuration and compatible endpoints

Rainy is the default backend. Configure an exact versioned base URL when using
another OpenAI-compatible service:

```rust,no_run
use rainy_sdk::{AuthConfig, RainyClient};

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let config = AuthConfig::new(std::env::var("API_KEY")?)
    .with_api_base_url("https://example-compatible-provider.com/v1")
    .with_timeout(60)
    .with_retry(false);
let client = RainyClient::with_config(config)?;
# let _ = client;
# Ok(())
# }
```

If `api_base_url` is not set, a root host is routed to `/api/v1`. A configured
base URL that already has a path, such as `/v1`, is used directly. HTTPS is
required by default; plain HTTP is accepted only for loopback test servers.

Authentication is selected by protocol. Chat, Responses, embeddings, and
Rainy extensions use `Authorization: Bearer ...`. Messages uses
`x-api-key: ...` and `anthropic-version: 2023-06-01`.

To target a compatible Messages deployment that requires another API version,
configure it on the client:

```rust,no_run
use rainy_sdk::{AuthConfig, RainyClient};

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = RainyClient::with_config(AuthConfig::new(std::env::var("API_KEY")?))?
    .with_anthropic_version("2024-10-22")?;
# let _ = client;
# Ok(())
# }
```

## Chat Completions

Use `ChatCompletionRequest` for compact text messages. Use
`OpenAIChatCompletionRequest` when replaying full tool-call history or sending
multimodal content:

```rust,no_run
use rainy_sdk::{
    FunctionDefinition, OpenAIChatCompletionRequest, OpenAIChatMessage, OpenAIContentPart,
    OpenAIMessageContent,
    ReasoningConfig, ResponseFormat, Tool, ToolType,
};

# fn example() -> rainy_sdk::Result<()> {
let request = OpenAIChatCompletionRequest::new(
    "compatible/chat-model",
    vec![OpenAIChatMessage::user(r#"Describe this image."#)],
)
.with_response_format(ResponseFormat::JsonObject)
.with_reasoning_config(ReasoningConfig::effort("high"))
.with_tools(vec![Tool {
    r#type: ToolType::Function,
    function: FunctionDefinition::new("lookup")
        .with_description("Look up a value")
        .with_parameters(rainy_sdk::serde_json::json!({
            "type": "object",
            "properties": {"key": {"type": "string"}}
        })),
}]);

let multimodal = OpenAIChatMessage::user(OpenAIMessageContent::parts(vec![
    OpenAIContentPart::text("This is text"),
]));
let _ = (request, multimodal);
# Ok(())
# }
```

`OpenAIContentPart` also provides image URL, input audio, and file variants.
Tool calls and tool results are retained as typed message fields, while
provider-specific replay metadata remains available through opaque extension
fields.

## Reasoning controls

OpenAI-compatible requests expose `reasoning`, `reasoning_effort`, and
`include_reasoning` directly. Stable effort values include `none`, `minimal`,
`low`, `medium`, `high`, `xhigh`, and `max`; unknown service-defined values can
be preserved with `ReasoningEffort::Custom` or a string conversion.

Effort and numeric budgets are intentionally different controls. The SDK never
converts an effort label into a guessed token count:

```rust,no_run
use rainy_sdk::{ChatCompletionRequest, ChatMessage, ReasoningConfig, RainyClient};

# fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let effort_request = ChatCompletionRequest::new(
    "compatible/reasoning-model",
    vec![ChatMessage::user("Think carefully, then answer.")],
)
.with_reasoning_effort("high")
.with_include_reasoning(true);

let explicit_budget = ChatCompletionRequest::new(
    "compatible/reasoning-model",
    vec![ChatMessage::user("Use an explicit budget.")],
)
.with_reasoning_config(ReasoningConfig::manual_budget(2048));

# let _ = (client, effort_request, explicit_budget);
# Ok(())
# }
```

## Responses

`ResponsesRequest` keeps the Responses wire shape open for built-in tools and
future-compatible input items:

```rust,no_run
use rainy_sdk::{RainyClient, ResponsesRequest};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let request = ResponsesRequest::text(
    "compatible/responses-model",
    "Summarize the supplied context.",
)
.with_reasoning_effort("medium")
.with_max_output_tokens(800)
.add_function_tool(
    "lookup",
    "Look up a value",
    rainy_sdk::serde_json::json!({"type": "object", "properties": {}}),
);

let response = client.create_response(request).await?.0;
println!("{}", response.text().unwrap_or_default());
# Ok(())
# }
```

Use `ResponsesRequest::function_call_output` together with
`with_previous_response_id` for a tool-call continuation. Responses streaming
returns `ResponsesEvent`, preserving the native event name and payload.

## Native Anthropic Messages

Messages has its own request and response types. Native thinking is represented
as `thinking.budget_tokens`; it is not translated into OpenAI reasoning:

```rust,no_run
use rainy_sdk::{AnthropicMessage, AnthropicMessageRequest, RainyClient};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let request = AnthropicMessageRequest::new(
    "compatible/messages-model",
    vec![AnthropicMessage::user("Give a concise answer.")],
    1024,
)
.with_system("You are concise.")
.with_thinking(256);

let response = client.create_message(request).await?;
println!("{}", response.text());
# Ok(())
# }
```

`create_message_stream` returns native Anthropic event objects such as
`message_start`, `content_block_delta`, and `message_stop`.

## Streaming

All protocol streams use one bounded incremental SSE parser. It handles
fragmented network chunks, CRLF/LF line endings, multiple `data:` lines,
`[DONE]`, and future event names without exposing credentials in parser errors:

```rust,no_run
use futures::StreamExt;
use rainy_sdk::{ChatCompletionRequest, ChatMessage, RainyClient};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let request = ChatCompletionRequest::new(
    "compatible/chat-model",
    vec![ChatMessage::user("Write a short haiku about Rust.")],
);
let mut stream = client.chat_completion_stream(request).await?;
while let Some(chunk) = stream.next().await {
    if let Some(delta) = chunk?.choices.first().and_then(|choice| choice.delta.content.as_deref()) {
        print!("{delta}");
    }
}
# Ok(())
# }
```

Use `chat_completion_stream_events` when billing or unknown named events are
useful. Standard inference POSTs are sent once; only explicitly safe discovery
operations use the configured retry policy.

## Embeddings

```rust,no_run
use rainy_sdk::{EmbeddingEncodingFormat, EmbeddingsRequest, RainyClient};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let request = EmbeddingsRequest::text("compatible/embedding-model", "Rainy embeddings")
    .with_encoding_format(EmbeddingEncodingFormat::Float)
    .with_dimensions(768);
let response = client.create_embeddings(request).await?;
println!("{} embedding(s)", response.data.len());
# Ok(())
# }
```

## Explicit Rainy extensions

Model discovery is an explicit opt-in operation, not an inference preflight:

```rust,no_run
use rainy_sdk::{ModelSelectionCriteria, RainyClient};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let models = client
    .select_models(ModelSelectionCriteria {
        required_input_modalities: vec!["text".to_string()],
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

The catalog helpers operate only on public model identifiers, architecture,
pricing, supported parameters, and generic capability flags. Health, search,
and Rainy response-envelope helpers are similarly additive extensions.

## Optional session/account APIs

`RainySessionClient` is not compiled into the default feature set. Enable
`rainy-account` when a separate application component needs JWT login, profile,
usage, or API-key management:

```rust,no_run
use rainy_sdk::RainySessionClient;

# async fn example() -> Result<(), Box<dyn std::error::Error>> {
let mut session = RainySessionClient::new()?;
let login = session
    .login(
        &std::env::var("RAINY_EMAIL")?,
        &std::env::var("RAINY_PASSWORD")?,
    )
    .await?;
println!("signed in as {}", login.user.email);
# Ok(())
# }
```

Session tokens are held in memory, redacted from the session client's `Debug`
output, and should never be logged or committed.

## Errors and security

Fallible methods return `Result<T, RainyError>`. Error bodies and successful
JSON responses are bounded, SSE frames are bounded, redirects are disabled,
and diagnostic messages avoid raw request URLs and credential values.

Use `RainyError::is_retryable` and `retry_after` for explicit recovery logic.
Do not automatically replay an inference POST: a retry can duplicate model work
or billing when the server accepted the original request.

## Development

See [`CONTRIBUTING.md`](CONTRIBUTING.md) for the contribution workflow and
[`SECURITY.md`](SECURITY.md) for security reports. The release validation matrix
includes minimal/default, no-default-feature, legacy, and all-feature checks.

```bash
cargo fmt --all -- --check
cargo check --locked --all-targets --all-features
cargo test --locked --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo doc --locked --all-features --no-deps
cargo package --locked --list
```

## License

Apache-2.0. See [LICENSE](LICENSE).
