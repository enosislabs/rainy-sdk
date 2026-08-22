# Repository Guidelines

## Project Shape and Boundaries

`rainy-sdk` is a single Rust library crate using edition 2024 and the pinned Rust 1.98.0 toolchain in [`rust-toolchain.toml`](rust-toolchain.toml). Public exports are assembled in [`src/lib.rs`](src/lib.rs).

- [`src/client.rs`](src/client.rs): `RainyClient`, API-key inference, shared HTTP/SSE transport, models, chat, responses, search, and health.
- [`src/session.rs`](src/session.rs): `RainySessionClient`, JWT authentication, account/org, usage, and key management.
- [`src/models.rs`](src/models.rs): public request/response and compatibility types.
- [`src/auth.rs`](src/auth.rs), [`src/error.rs`](src/error.rs), [`src/retry.rs`](src/retry.rs): API-key validation, error taxonomy, and backoff.
- [`src/search.rs`](src/search.rs): search/research types and compatibility mapping.
- [`src/endpoints/`](src/endpoints/): endpoint-specific extensions, including chat, search, health, and legacy account helpers.

Keep the client split intact: use `RainyClient` for API-key runtime operations and `RainySessionClient` for JWT/dashboard operations. Do not add new account, usage, or key flows to the API-key client. See [`MIGRATION.md`](MIGRATION.md) for the v2-to-v3 mapping.

Route through existing helpers rather than hardcoding URLs:

- `root_url(...)` / `root_request(...)` for host-level routes such as `/health`.
- `api_v1_url(...)` / `api_request(...)` for `/api/v1/*` routes.
- Session routes independently use the `/api/v1/*` namespace.

## Build and Validation

Run focused tests while iterating, then the relevant full checks before finishing:

- `cargo build` and `cargo test`
- `cargo build --all-features` and `cargo test --all-features`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo doc --all-features --no-deps`
- For public API changes, `cargo test --doc --all-features`

CI also checks missing docs and broken intra-doc links, unused dependencies, security audit, and release builds. See [`.github/workflows/ci.yml`](.github/workflows/ci.yml) and [`.github/workflows/README.md`](.github/workflows/README.md) for the authoritative workflow details.

Useful test configuration:

- `RAINY_TEST_API_KEY`: API key for live/integration flows.
- `RAINY_TEST_BASE_URL`: test server override, commonly `http://localhost:3000`.
- Examples may also require `RAINY_EMAIL` and `RAINY_PASSWORD`.

The contributor guide references `.env.example`, but that file is not currently present. Never commit `.env`, credentials, API keys, refresh tokens, or session tokens.

## Implementation Conventions

- Use standard Rust naming: `snake_case` for modules/functions/tests, `PascalCase` for types, and `SCREAMING_SNAKE_CASE` for constants.
- Prefer consuming builder APIs such as `ChatCompletionRequest::new(...).with_temperature(...)`.
- Public fallible methods return `Result<T, RainyError>` and public APIs need `///` documentation; use `rust,no_run` examples for user-facing async APIs.
- Preserve serde aliases, `skip_serializing_if`, flattened forward-compatible fields, and provider-specific JSON shapes when extending models.
- Keep API keys in `secrecy::SecretString`; never log or expose secrets.
- Use Tokio async tests and `mockito` for HTTP mocking. Name tests after observable behavior.
- Streaming methods force `stream = true`; retries cover only the initial connection, not partial streams. Preserve typed chunk, billing, and raw SSE event handling.
- The `legacy` feature is opt-in and gates legacy types/model constants and old key/usage/user helpers. When changing feature-sensitive code, validate both default and `--no-default-features` builds where practical.
- The crate metadata says edition 2024; do not “fix” the existing `.rustfmt.toml` edition setting without checking toolchain and CI implications.

When changing an endpoint, cover successful canonical/alias deserialization, request serialization, an error or fallback path, and feature-gated behavior where applicable. Add focused tests near the matching suite in [`tests/`](tests/): unit behavior, OpenAI chat/SSE, Responses/catalog, session auth, or search/research.

## Documentation and Contributions

Link to existing documentation instead of duplicating it:

- [`README.md`](README.md): setup, API examples, features, and architecture overview.
- [`CONTRIBUTING.md`](CONTRIBUTING.md): development workflow, testing, DCO, and pull requests.
- [`MIGRATION.md`](MIGRATION.md): client split and v2-to-v3 migration.
- [`docs/GEMINI_3_INTEGRATION.md`](docs/GEMINI_3_INTEGRATION.md): Gemini thinking and thought signatures.

Use imperative Conventional Commit subjects and sign commits with DCO (`git commit -s`). User-visible changes should update [`CHANGELOG.md`](CHANGELOG.md), documentation, and tests as appropriate. Report security issues through [`SECURITY.md`](SECURITY.md), not public issues.
