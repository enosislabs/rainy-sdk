# Rainy API capability matrix

[`API_CAPABILITY_MATRIX.json`](API_CAPABILITY_MATRIX.json) is the machine-readable route inventory shipped with the SDK. The same data is exported at runtime as `rainy_sdk::capability_matrix()` and `rainy_sdk::RAINY_API_CAPABILITY_MATRIX`.

Each row records the HTTP method, source route or route family, ownership class, authentication scheme, SDK support status, and whether the route uses Server-Sent Events. `implemented` means the SDK has a typed operation; `known_unsupported` records a route that the API explicitly rejects; `not_exposed` records a real service route intentionally outside the stable public SDK surface.

The matrix is tested against the Rust constant so the packaged documentation cannot silently drift from the code. It is a route classification, not permission to call dashboard, internal, browser-auth, webhook, telemetry, training, or integration routes directly.
