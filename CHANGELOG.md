# Changelog

All notable changes to `rainy-sdk` are documented here. The project follows
semantic-versioning conventions where practical.

## [Unreleased] - 2026-08-30

### Changed

- Reorganized the public SDK around OpenAI-compatible Chat Completions,
  Responses, embeddings, and native Anthropic Messages protocols.
- Reused one bounded incremental SSE parser for all supported streaming
  protocols, including fragmented chunks, CRLF/LF framing, multiple data
  lines, completion sentinels, and unknown event names.
- Made reasoning effort a typed wire value and kept explicit numeric budgets
  separate from effort labels. No effort-to-budget conversion is performed.
- Made account/session APIs opt-in through `rainy-account`; the default
  inference build remains API-key and protocol focused.
- Reduced default features to an empty set. Rate limiting and tracing remain
  available as explicit additive features.

### Security

- Added protocol-specific authentication headers for Bearer and native
  `x-api-key`/version requests.
- Disabled redirects for authenticated requests and restricted plain HTTP to
  loopback test URLs.
- Added bounded request, response, error, and SSE frame handling.
- Redacted credentials from client and session diagnostics and avoided raw
  request URLs in error messages.
- Inference POST requests are sent once; only explicitly safe operations use
  automatic retry behavior.

### Removed

- Removed obsolete provider-specific model constants, thinking-level
  validators, and provider-era request architecture.
- Removed the obsolete provider-specific thinking example and private
  capability inventories from the package surface.

## [0.6.16]

- Initial public Rainy API v3 SDK release line.
- Added asynchronous typed clients, health and search helpers, model-list
  compatibility types, retry configuration, and legacy compatibility gates.

[Unreleased]: https://github.com/enosislabs/rainy-sdk/compare/v0.6.16...HEAD
[0.6.16]: https://github.com/enosislabs/rainy-sdk/releases/tag/v0.6.16
