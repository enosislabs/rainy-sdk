# Rainy SDK v2 -> v3 Migration Guide

This guide helps migrate SDK usage from legacy v2-era patterns to the Rainy API v3 service.

Important:

- The Rainy API v3 service currently exposes canonical HTTP routes under `/api/v1/*`.
- Legacy `cowork` routes were removed due to billing/usage discrepancies and operational overhead.
- Traces of Cowork remain in the SDK only as deprecated compatibility (opt-in `cowork` feature).

## Client Split (Recommended)

Use the SDK by trust boundary:

- `RainyClient` (API key / `ra-*`): runtime inference and search
  - models
  - chat completions
  - responses
  - search
- `RainySessionClient` (JWT session): dashboard/account operations
  - auth
  - keys
  - usage
  - org profile
  - users

This split reduces accidental misuse of JWT-only endpoints with API keys and keeps the default SDK surface smaller.

## Quick Mapping (Methods)

### Runtime / API Key (keep using `RainyClient`)

- `RainyClient::get_available_models()` -> `RainyClient::get_available_models()` (v3-mapped)
- `RainyClient::list_available_models()` -> `RainyClient::list_available_models()` (alias; v3-mapped)
- `RainyClient::chat_completion(...)` -> `RainyClient::chat_completion(...)` (v3 `/api/v1/chat/completions`)
- `RainyClient::create_chat_completion(...)` -> `RainyClient::create_chat_completion(...)` (v3 `/api/v1/chat/completions`)
- `RainyClient::research(...)` -> `RainyClient::research(...)` (now mapped to v3 `/api/v1/search`)

Notes:

- `research(...)` now uses the v3 search endpoint and synthesizes a legacy-compatible summary payload.
- Health endpoints are root routes in v3 (`/health`, `/health/dependencies`), and the SDK now calls the correct paths.

### Deprecated v2-Style Methods (move to `RainySessionClient`)

- `RainyClient::get_user_account()` (deprecated)
  - Replace with:
    - `RainySessionClient::me()`
    - `RainySessionClient::org_me()` as needed

- `RainyClient::list_api_keys()` / `create_api_key()` / `update_api_key()` / `delete_api_key()` (deprecated)
  - Replace with:
    - `RainySessionClient::list_api_keys()`
    - `RainySessionClient::create_api_key(...)`
    - `RainySessionClient::delete_api_key(...)`
  - Note: `update_api_key()` is not reintroduced yet to keep the v3 surface minimal. Use direct HTTP if needed.

- `RainyClient::get_credit_stats()` / `get_usage_stats()` (deprecated)
  - Replace with:
    - `RainySessionClient::usage_credits()`
    - `RainySessionClient::usage_stats(days)`

## Cowork Legacy Compatibility (Opt-In Only)

Cowork-related SDK compatibility is no longer enabled by default.

To compile Cowork compatibility traces:

```toml
[dependencies]
rainy-sdk = { version = "0.6.4", features = ["cowork"] }
```

Use only for short-lived migration support. New integrations should not depend on Cowork endpoints.

## Code Examples

### Before (v2-style, mixed concerns)

```rust,no_run
use rainy_sdk::RainyClient;

let client = RainyClient::with_api_key("ra-...")?;
let _user = client.get_user_account().await?; // deprecated
let _keys = client.list_api_keys().await?; // deprecated
```

### After (v3 split)

```rust,no_run
use rainy_sdk::{RainyClient, RainySessionClient};

let api_client = RainyClient::with_api_key("ra-...")?;
let _models = api_client.get_available_models().await?;

let mut session = RainySessionClient::new()?;
let login = session.login("user@example.com", "password").await?;
let _org = session.org_me().await?;
let _credits = session.usage_credits().await?;
```

## Production Migration Checklist

- Update to a version containing the v3 base URL and route fixes
- Stop using deprecated `RainyClient` account/keys/usage helpers
- Move JWT/dashboard flows to `RainySessionClient`
- Remove any Cowork feature dependency unless strictly required for migration
- Run tests with:
  - default features
  - `--no-default-features`
  - (optional) `--features cowork` if legacy compatibility must remain enabled

## Security/Operational Guidance

- Keep JWT session tokens in trusted environments only (backend services or trusted desktop apps).
- Do not expose session/admin tokens in browser-distributed applications.
- Prefer the smallest SDK client surface needed for each component (`RainyClient` vs `RainySessionClient`).
