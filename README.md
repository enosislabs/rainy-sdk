# Rainy SDK

Rust SDK for the Rainy API v3.8.1 service. It separates API-key runtime operations (`RainyClient`) from JWT dashboard/account operations (`RainySessionClient`).

## Installation

```toml
[dependencies]
rainy-sdk = "0.6.15"
```

The crate uses Rust 1.98.0 and Rustls. The optional `legacy` feature exposes compatibility types and older account helpers.

## Authentication

```rust,no_run
use rainy_sdk::RainyClient;

# fn example() -> Result<(), Box<dyn std::error::Error>> {
let client = RainyClient::with_api_key(std::env::var("RAINY_API_KEY")?)?;
# Ok(())
# }
```

Use `RainyClient` for chat, Responses, embeddings, search, health, and model discovery. Use `RainySessionClient` with a JWT for account, organization, usage, and API-key management.

## Model discovery and access

Do not hardcode a model inventory: the catalog changes independently of the crate. Catalog presence does not guarantee that an organization can call a model. Effective access can depend on model tier, plan, purchased credits, model allowlists, organization policy, privacy, capability entitlements, and context limits.

Request the catalog with an API key so organization-specific fields are populated:

```rust,no_run
use rainy_sdk::{ModelSelectionCriteria, ModelTier, RainyClient};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let models = client
    .select_models(ModelSelectionCriteria {
        allowed_tiers: vec![ModelTier::Core, ModelTier::Standard],
        require_tools: Some(true),
        require_organization_access: true,
        ..Default::default()
    })
    .await?;

for model in models {
    println!("{}: {:?}", model.id, model.rainy_model_tier);
}
# Ok(())
# }
```

`require_organization_access` checks organization-enabled and privacy-compatible flags. Anonymous catalog entries are excluded because their usability cannot be determined accurately.

## Chat completions

```rust,no_run
use rainy_sdk::{ChatCompletionRequest, ChatMessage, RainyClient};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let request = ChatCompletionRequest::new(
    "provider/model-id",
    vec![ChatMessage::user("Explain ownership in Rust")],
)
.with_max_tokens(500)
.with_temperature(0.2);

let (response, metadata) = client.chat_completion(request).await?;
println!("{}", response.choices[0].message.content);
println!("request metadata: {metadata:?}");
# Ok(())
# }
```

For OpenAI-compatible tool-call history and multimodal messages, use `OpenAIChatCompletionRequest`. Streaming exposes typed chunk, billing, and raw SSE events.

## Responses API

```rust,no_run
use rainy_sdk::{RainyClient, ResponsesRequest};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let response = client
    .create_response(
        ResponsesRequest::text("provider/model-id", "Summarize this document")
            .with_reasoning_effort("medium")
            .with_max_output_tokens(800),
    )
    .await?;
println!("{response:?}");
# Ok(())
# }
```

## Embeddings

The embeddings surface accepts strings or token arrays, optional dimensions, and float or base64 output.

```rust,no_run
use rainy_sdk::{EmbeddingEncodingFormat, EmbeddingsRequest, RainyClient};

# async fn example(client: &RainyClient) -> rainy_sdk::Result<()> {
let mut request = EmbeddingsRequest::text(
    "provider/embedding-model-id",
    "Rainy embeddings",
);
request.encoding_format = Some(EmbeddingEncodingFormat::Float);
request.dimensions = Some(768);

let response = client.create_embeddings(request).await?;
println!("{} embedding(s)", response.data.len());
# Ok(())
# }
```

The model must declare embedding output support and the organization must have embedding and model access. Entitlement failures are returned as `RainyError::AccessDenied` with the original code and structured details.

## Development

The pinned toolchain and committed lockfile are authoritative. See [CONTRIBUTING.md](CONTRIBUTING.md), [CHANGELOG.md](CHANGELOG.md), and [SECURITY.md](SECURITY.md).

```text
cargo fmt --all -- --check
cargo check --locked --all-targets --all-features
cargo test --locked --all-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo doc --locked --all-features --no-deps
```

## License

Apache-2.0. See [LICENSE](LICENSE).
