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

## Non-Negotiable Quality Gates

These rules are release-blocking. A green local command is not evidence of success if it is not the same command CI runs.

- Never declare a change complete while `git status --short --untracked-files=all` reports anything. Inspect and resolve every modified, deleted, and untracked file before committing; never hide generated files with `.gitignore` just to obtain a clean status.
- Always run `git diff --check` and `cargo fmt --all -- --check` before committing. Formatting, whitespace, and merge-marker failures are defects, not cosmetic warnings.
- Before changing implementation code, establish a compiling baseline with `cargo check --locked --all-targets --all-features`. If the worktree is already broken, record the exact errors, preserve existing user changes, repair parsing/build blockers first, and add regression tests for every restored branch.
- Do not use `cargo update --locked` as a compatibility check. `cargo update` attempts to change the lockfile and is expected to fail with `--locked`; use `cargo check --locked --all-targets --all-features` to verify the committed lockfile. Regenerate `Cargo.lock` only after `Cargo.toml` and source code compile, then review the complete lockfile diff and run the security audit.
- Any dependency refresh must run `cargo machete` (if installed), `cargo audit`, and the complete feature matrix. Do not remove a direct dependency based only on a source search; verify examples, tests, build scripts, optional features, and platform targets.
- Do not weaken, bypass, or dynamically rewrite CI checks to make a run green. If a lint policy is intentionally changed, reproduce the original failure, document the new policy in the workflow, retain `-D warnings`, and run the exact resulting command locally. Never let local validation and CI validate different code or lint rules.
- Do not generate benchmark files, edit `Cargo.toml` dynamically, or rely on untracked CI fixtures. Benchmarks and their dependencies must be tracked, reproducible, and runnable from a clean checkout.

The minimum release validation matrix is:

```text
cargo fmt --all -- --check
cargo check --locked --all-targets --all-features
cargo check --locked --no-default-features
cargo check --locked --features legacy
cargo test --locked --all-features
cargo test --locked --no-default-features
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo doc --locked --all-features --no-deps
cargo build --locked --release --all-features
cargo bench --bench client_benchmark
cargo package --locked --list
```

Run the same feature matrix in CI. For release-sensitive changes, also run strict documentation checks with `RUSTDOCFLAGS='-D warnings -D missing_docs -D rustdoc::broken_intra_doc_links'` and verify the package with `cargo package --locked`.

## Feature, Example, and API Contract Rules

- Every README, doctest, generated example, and test-readme crate must compile under the feature set it claims to support. Default examples may import only default-feature exports; legacy-only types such as `ChatRole` require an explicit `legacy` feature or must be replaced with the default `MessageRole` API.
- When adding or moving a public export, search all README examples, `examples/`, documentation generators, doctests, integration fixtures, and re-exports. A successful library build does not prove that examples compile.
- Every feature-gated branch must have a focused test for both the enabled and disabled behavior where practical. Do not restore or remove validation branches—thinking levels, zero-count handling, Gemini validation, billing events, aliases, and fallbacks—without a regression test.
- Preserve public names, signatures, serde aliases, wire formats, retry behavior, TLS requirements, and `ChatStreamEvent` variants unless the task explicitly authorizes an API break. Native SSE event names must be preferred before payload-shape fallback; test chunk, billing, raw, `[DONE]`, malformed, and stream-error events.
- Keep private implementation repositories, internal service names, and non-public contract sources out of this public SDK. Use external contract references only through private local context; never name or link those repositories in tracked files, documentation, examples, logs, commits, or release notes.

## Dependency, Lockfile, and Security Rules

- Rust and Cargo are pinned together through [`rust-toolchain.toml`](rust-toolchain.toml). Do not silently use another toolchain for release decisions; record `rustc --version`, `cargo --version`, and the lockfile state when diagnosing a build.
- `Cargo.lock` is part of the release artifact. It must be committed, accepted by `cargo check --locked`, and regenerated only through Cargo. Never hand-edit dependency checksums or delete the lockfile to make resolution pass.
- After a lockfile refresh, inspect direct and transitive changes, run `cargo audit`, and confirm known advisories are resolved before pushing. A GitHub dependency warning or failed security job is a release blocker, even if compilation succeeds.
- Keep optional dependencies behind the correct feature and keep production feature flags narrower than test-only features. Verify `--all-features`, `--no-default-features`, and each named compatibility feature independently.
- Never print, commit, echo, or paste registry tokens, API keys, refresh tokens, or secret environment variables. Do not use shell tracing (`set -x`) around credential operations.

## Release and Tagging Rules

Treat a release as a transaction with explicit preflight, push, verification, and post-release phases.

### Preflight

- Release only from the intended branch and a clean worktree. Confirm `git branch --show-current`, `git status --short --untracked-files=all`, and `git log -1 --oneline` before tagging.
- Confirm the crate version, changelog heading, README/toolchain version references, workflow versions, and release tag all match exactly. The tag `vX.Y.Z` must match `Cargo.toml` version `X.Y.Z`; never tag a `-preview` or stale commit as a final release.
- Run the complete validation matrix, `cargo audit`, `cargo package --locked --list`, and the tracked benchmark before creating the tag. Verify the package contains the intended source, docs, license, changelog, and no secrets.
- Confirm the target commit is already present on the remote branch before tagging. A tag must never point to an unpushed or dirty commit.
- Before triggering a release workflow, verify the required secret names exist with `gh secret list`. Secret existence does not prove validity; the crates.io token must be known to be current and authorized for the crate. If it cannot be verified, stop before tagging and ask the release owner to rotate it securely.

### Tag creation and push

- Use an annotated tag with the exact release message: `git tag -a vX.Y.Z -m "vX.Y.Z is out!"`.
- A Git remote name and a tag name are different arguments. Never run `git push tag vX.Y.Z`, `git push vX.Y.Z`, or `git push tag`; those commands interpret the tag text as a remote. Push explicitly with `git push origin refs/tags/vX.Y.Z`.
- Push the commit and tag explicitly: `git push origin HEAD:refs/heads/main` followed by `git push origin refs/tags/vX.Y.Z`. Prefer a normal fast-forward push. Do not use `--force` for a release tag unless the release owner explicitly authorizes retagging after reviewing the exact old and new peeled commits.
- If `refs/tags/vX.Y.Z` already exists, stop and compare `git rev-parse vX.Y.Z^{}` with the intended commit. Never overwrite an existing release tag to repair a failed build; publish a new patch version unless the owner explicitly authorizes a retag. Crates.io versions and GitHub releases are immutable in practice.

### Remote and workflow verification

- Immediately verify the remote refs, not just the local tag:

  ```text
  git ls-remote origin refs/heads/main refs/tags/vX.Y.Z 'refs/tags/vX.Y.Z^{}'
  ```

  The branch and peeled tag must resolve to the intended commit. For an annotated tag, the tag-object hash and peeled commit hash are expected to differ.
- Monitor every workflow triggered by the commit/tag with `gh run list` or the GitHub Actions UI. Inspect failed logs with `gh run view RUN_ID --log-failed`; do not infer success from one green job or from the tag existing.
- A release is not complete until validation, formatting, Clippy, all feature tests, docs, security audit, performance checks, both platform builds/tests, crates.io publication, and GitHub Release creation all report success.
- Distinguish GitHub tag creation from package publication. Verify `https://crates.io/api/v1/crates/rainy-sdk` contains the exact version and verify the GitHub Release has the expected notes and `.crate` asset. A tag alone is not a published SDK.
- If a release job fails, identify the root cause from the job log before rerunning. Do not blindly rerun, retag, or create a second release. A `403 authentication failed` from crates.io is a credentials/secrets incident: rotate `CRATES_IO_TOKEN` through GitHub Secrets, then rerun the workflow; never paste the token into chat, source, or command history.
- Do not report “released,” “published,” or “uploaded” until the final remote refs and all release artifacts have been independently verified. Report partial completion and the exact blocker when any gate fails.

## Benchmark and Performance Rules

- Benchmark the tracked baseline and the changed code on the same machine, toolchain, feature set, and lockfile. Record client construction, serialization, JSON parsing, native SSE chunk/billing/raw parsing, release-build time, and artifact size.
- A single noisy Criterion run is not a regression verdict. If a result crosses the acceptance threshold or Criterion reports a significant change, rerun it at least once under the same conditions and investigate before claiming improvement.
- The release target is no runtime regression greater than 5%, no cold release-build regression greater than 10%, and no artifact-size increase greater than 5%, unless the release notes explicitly document and approve the tradeoff. Never reset or delete Criterion baselines to hide a regression.
- Keep optimized transport changes private unless the public API requires otherwise. Preserve memoized clients, TLS 1.2 minimum, HTTPS-only behavior, retry semantics, request IDs, rate limits, and streaming error handling while optimizing.

## Mandatory Handoff Checklist

Before handing work back, include the exact commit SHA, tag target, commands run, benchmark result, package result, workflow URLs/statuses, and any unresolved blocker. The final response must state plainly whether the work is fully released, only pushed to GitHub, or blocked before publication.
