# Repository Guidelines

## Project Structure & Architecture

This repository is a single Rust library crate, `rainy-sdk`. Public exports are assembled in `src/lib.rs`. The core transport layer lives in `src/client.rs` and `src/session.rs`:

- `RainyClient`: API-key client for model, chat, responses, search, and health endpoints.
- `RainySessionClient`: JWT/session client for dashboard-style endpoints such as auth, orgs, usage, and keys.

Domain types live in `src/models.rs`, authentication setup in `src/auth.rs`, retry behavior in `src/retry.rs`, and search/research types in `src/search.rs`. Endpoint-specific request methods are split under `src/endpoints/` (`chat.rs`, `search.rs`, `health.rs`, `keys.rs`, `usage.rs`, `user.rs`, and feature-gated `cowork.rs`).

Routing is centralized through helper methods on `RainyClient`:

- `root_url(...)` targets host-level routes such as `/health`
- `api_v1_url(...)` targets versioned API routes such as `/api/v1/chat/completions`

Keep new endpoint methods consistent with that split instead of hardcoding URLs in multiple places.

## Build, Test, and Dev Commands

- `cargo build`: compile the crate with default features.
- `cargo build --all-features`: verify optional surfaces such as `cowork`.
- `cargo test`: run unit, integration, and doc tests.
- `cargo test --test session_client_integration_test -- --nocapture`: run one suite with visible output.
- `cargo fmt --check`: enforce formatting.
- `cargo clippy --all-targets --all-features`: catch lint and API consistency issues.
- `cargo doc --no-deps`: build public docs locally.

Useful env vars for tests:

- `RAINY_TEST_API_KEY`: test API key for client integration flows.
- `RAINY_TEST_BASE_URL`: override base URL, usually `http://localhost:3000`.

## Coding Style & Naming Conventions

Use Rust 2021 idioms and `rustfmt` defaults with 4-space indentation. Follow these naming rules:

- modules, functions, and tests: `snake_case`
- structs, enums, and traits: `PascalCase`
- constants: `SCREAMING_SNAKE_CASE`

Prefer builder-style APIs for request types, matching existing patterns like `ChatCompletionRequest::new(...).with_temperature(...)`. Public methods should return `Result<T, RainyError>` and keep serialization details encapsulated inside the client layer. Document public API surfaces with `///` doc comments and add `rust,no_run` examples where the behavior is user-facing.

## Testing Guidelines

The repository uses:

- inline/unit coverage in `tests/unit_tests.rs`
- integration coverage in `tests/*.rs`
- async tests with `#[tokio::test]`
- HTTP mocking with `mockito`

Name tests after observable behavior, for example `session_org_and_usage_calls_send_bearer_token`. When adding or changing endpoints, cover:

- successful deserialization of canonical and alias payloads
- request serialization for new fields
- at least one error or fallback path
- feature-gated behavior where applicable

If you add public request/response types, also add a focused serialization test similar to `tests/responses_api_test.rs`.

## Commit & Pull Request Guidelines

Recent history uses Conventional Commits, including scoped forms:

- `feat: add responses api compatibility for rainy v3`
- `feat(api): migrate to Rainy API v3 with session-based authentication`
- `chore: release v0.6.9`

Use imperative subjects, add scope when it clarifies the subsystem, and sign commits with DCO: `git commit -s -m "feat(api): ..."`.

Pull requests should include:

- a short summary of behavior changes
- the exact commands run (`cargo test`, `cargo clippy`, etc.)
- documentation updates when public APIs or examples changed
- `CHANGELOG.md` updates for user-visible changes
- sample request/response notes when API contracts changed

## Security & Configuration Tips

Do not commit real API keys, refresh tokens, or session tokens. Keep secrets in environment variables and prefer mock servers for endpoint tests. For security issues, follow the private reporting path in `SECURITY.md` instead of opening a public issue.
