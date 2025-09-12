## [1.0.0] - 2025-09-12

### Security
- **API Key Hardening**: The client now uses the `secrecy` crate to handle the API key. The key is stored in a protected memory region and is securely zeroed out when the client is dropped, reducing the risk of key leakage from memory.
- **TLS Hardening**: The underlying HTTP client has been hardened to use `rustls` as the TLS backend, and it now enforces TLS 1.2+ and HTTPS-only connections. This provides stronger protection against network interception and downgrade attacks.
- **Improved Documentation**: Added a "Security Considerations" section to the `README.md` to clearly communicate the security posture of the SDK, including the purpose of client-side rate limiting and best practices for API key management.

### Changed
- The version number has been updated to `1.0.0` to signify a stable, production-ready release.
