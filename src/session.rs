//! Optional JWT/session client for Rainy account endpoints.
//!
//! This module is deliberately behind the non-default `rainy-account` feature.
//! It keeps session and dashboard-style operations separate from the
//! API-key-authenticated inference client and shares the SDK's URL, body-size,
//! redirect, and diagnostic-safety rules.

use crate::auth::{
    GENERAL_REQUEST_BODY_BYTES, MAX_ERROR_BODY_BYTES, MAX_RESPONSE_BODY_BYTES,
    read_limited_response_body, safe_url_label, serialize_json_body, validate_service_url,
};
use crate::error::{ApiErrorResponse, RainyError, Result};
use reqwest::{
    Client, Method, Response,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT},
};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::fmt;

/// Configuration for [`RainySessionClient`].
#[derive(Clone)]
pub struct SessionConfig {
    /// Base URL of the Rainy API v3 service (host only; API paths are added by the client).
    pub base_url: String,
    /// HTTP timeout in seconds for session requests.
    pub timeout_seconds: u64,
    /// User-Agent header used for session requests.
    pub user_agent: String,
}

impl fmt::Debug for SessionConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SessionConfig")
            .field("base_url", &safe_url_label(&self.base_url))
            .field("timeout_seconds", &self.timeout_seconds)
            .field("user_agent", &self.user_agent)
            .finish()
    }
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            base_url: crate::DEFAULT_BASE_URL.to_string(),
            timeout_seconds: 30,
            user_agent: format!("rainy-sdk/{}/session", crate::VERSION),
        }
    }
}

impl SessionConfig {
    /// Creates a session configuration with sane defaults for Rainy API v3.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets a custom base URL for the Rainy API service.
    pub fn with_base_url(mut self, base_url: impl Into<String>) -> Self {
        self.base_url = base_url.into();
        self
    }

    /// Sets a custom request timeout (seconds).
    pub fn with_timeout(mut self, timeout_seconds: u64) -> Self {
        self.timeout_seconds = timeout_seconds;
        self
    }

    /// Sets a custom User-Agent header value.
    pub fn with_user_agent(mut self, user_agent: impl Into<String>) -> Self {
        self.user_agent = user_agent.into();
        self
    }
}

/// Client for Rainy API v3 JWT/session endpoints.
///
/// Use this client for authentication and account operations such as
/// `/api/v1/auth/*`, `/api/v1/keys`, `/api/v1/usage/*`, and `/api/v1/orgs/me`.
/// It is not part of the default inference client or default feature set.
pub struct RainySessionClient {
    client: Client,
    config: SessionConfig,
    access_token: Option<SecretString>,
}

/// Request body for `POST /api/v1/auth/login`.
#[derive(Debug, Clone, Serialize)]
pub struct LoginRequest<'a> {
    /// User email address.
    pub email: &'a str,
    /// User password.
    pub password: &'a str,
}

/// Request body for `POST /api/v1/auth/register`.
#[derive(Debug, Clone, Serialize)]
pub struct RegisterRequest<'a> {
    /// User email address.
    pub email: &'a str,
    /// User password.
    pub password: &'a str,
    /// Region code (for example `us` or `la`).
    pub region: &'a str,
}

/// Request body for `POST /api/v1/auth/refresh`.
#[derive(Debug, Clone, Serialize)]
pub struct RefreshRequest<'a> {
    /// Refresh token issued by the auth endpoints.
    #[serde(rename = "refreshToken")]
    pub refresh_token: &'a str,
}

/// Authenticated user profile returned by session endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    /// User identifier.
    pub id: String,
    /// User email.
    pub email: String,
    /// User role (`admin`, `member`, etc.).
    pub role: String,
    /// Organization identifier when included by the endpoint.
    #[serde(rename = "orgId", default)]
    pub org_id: Option<String>,
}

/// Pair of access and refresh tokens.
#[derive(Clone, Serialize, Deserialize)]
pub struct SessionTokens {
    /// Access token for authenticated session requests.
    #[serde(rename = "accessToken")]
    pub access_token: String,
    /// Refresh token used to renew the access token.
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

/// Response payload for login/register auth endpoints.
#[derive(Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    /// Access token returned by the API.
    #[serde(rename = "accessToken")]
    pub access_token: String,
    /// Refresh token returned by the API.
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
    /// Authenticated user information.
    pub user: SessionUser,
}

/// Response payload for token refresh.
#[derive(Clone, Serialize, Deserialize)]
pub struct RefreshResponse {
    /// New access token.
    #[serde(rename = "accessToken")]
    pub access_token: String,
    /// Rotated or reissued refresh token.
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

/// Organization profile returned by `GET /api/v1/orgs/me`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgProfile {
    /// Organization identifier.
    pub id: String,
    /// Organization display name.
    pub name: String,
    /// Plan identifier (`payg`, `teams`, etc.).
    #[serde(rename = "planId")]
    pub plan_id: String,
    /// Organization region code.
    pub region: String,
    /// ISO-8601 creation timestamp.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// Current credit balance as a string (server preserves precision).
    pub credits: String,
}

/// API key summary item returned by `GET /api/v1/keys`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionApiKeyListItem {
    /// API key identifier.
    pub id: String,
    /// User-defined key name.
    pub name: String,
    /// Key type if returned by the endpoint (`standard`, `platform`).
    #[serde(default)]
    pub r#type: Option<String>,
    /// Whether the key is active.
    #[serde(rename = "isActive")]
    pub is_active: bool,
    /// Last-used timestamp when available.
    #[serde(rename = "lastUsed", default)]
    pub last_used: Option<String>,
    /// Creation timestamp.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// Masked prefix for display purposes.
    #[serde(default)]
    pub prefix: Option<String>,
}

/// Created API key response for `POST /api/v1/keys`.
#[derive(Clone, Serialize, Deserialize)]
pub struct CreatedApiKey {
    /// Plaintext API key value (returned only at creation time).
    pub key: String,
    /// Key identifier.
    pub id: String,
    /// Key display name.
    pub name: String,
    /// Key type (`standard` or `platform`).
    pub r#type: String,
}

/// Credits balance response for `GET /api/v1/usage/credits`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageCreditsResponse {
    /// Current credit balance.
    pub balance: f64,
    /// Currency unit (typically `credits`).
    pub currency: String,
    /// Source metadata alias returned by the server.
    #[serde(default)]
    pub source: Option<String>,
}

/// Usage statistics response for `GET /api/v1/usage/stats`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatsResponse {
    /// Number of days included in the aggregation window.
    #[serde(rename = "periodDays")]
    pub period_days: u32,
    /// Total requests in the selected period.
    #[serde(rename = "totalRequests")]
    pub total_requests: u64,
    /// Total credits deducted in the selected period.
    #[serde(rename = "totalCreditsDeducted")]
    pub total_credits_deducted: f64,
    /// Generic usage summary returned by the service.
    #[serde(rename = "statsByProvider", default)]
    pub stats_by_provider: serde_json::Value,
    /// Recent usage logs alias payload.
    #[serde(default)]
    pub logs: Vec<serde_json::Value>,
    /// Canonical envelope `data` field when preserved by deserialization.
    #[serde(default)]
    pub data: Option<serde_json::Value>,
}

#[derive(Debug, Deserialize)]
struct ApiEnvelope<T> {
    success: bool,
    data: T,
}

#[derive(Debug, Deserialize)]
struct ListKeysEnvelope {
    success: bool,
    keys: Vec<SessionApiKeyListItem>,
}

impl RainySessionClient {
    /// Creates a session client using default configuration.
    pub fn new() -> Result<Self> {
        Self::with_config(SessionConfig::default())
    }

    /// Creates a session client with custom configuration.
    pub fn with_config(config: SessionConfig) -> Result<Self> {
        validate_service_url(&config.base_url, "INVALID_BASE_URL")?;
        if config.timeout_seconds == 0 {
            return Err(RainyError::InvalidRequest {
                code: "INVALID_TIMEOUT".to_string(),
                message: "session timeout must be greater than zero".to_string(),
                details: None,
            });
        }

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&config.user_agent).map_err(|_| RainyError::InvalidRequest {
                code: "INVALID_USER_AGENT".to_string(),
                message: "User-Agent contains invalid header characters".to_string(),
                details: None,
            })?,
        );

        let client = Client::builder()
            .tls_backend_rustls()
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .default_headers(headers)
            .build()
            .map_err(|_| RainyError::Network {
                message: "Failed to create HTTP client".to_string(),
                retryable: false,
                source_error: None,
            })?;

        Ok(Self {
            client,
            config,
            access_token: None,
        })
    }

    /// Creates a session client with only a custom base URL override.
    pub fn with_base_url(base_url: impl Into<String>) -> Result<Self> {
        Self::with_config(SessionConfig::default().with_base_url(base_url))
    }

    /// Sets the in-memory access token used for authenticated requests.
    pub fn set_access_token(&mut self, access_token: impl Into<String>) {
        self.access_token = Some(SecretString::from(access_token.into()));
    }

    /// Clears the in-memory access token.
    pub fn clear_access_token(&mut self) {
        self.access_token = None;
    }

    /// Returns the current in-memory access token, if set.
    pub fn access_token(&self) -> Option<&str> {
        self.access_token
            .as_ref()
            .map(|token| token.expose_secret())
    }

    /// Returns the configured API base URL.
    pub fn base_url(&self) -> &str {
        &self.config.base_url
    }

    fn api_v1_url(&self, path: &str) -> String {
        let normalized = if path.starts_with('/') {
            path.to_string()
        } else {
            format!("/{path}")
        };
        format!(
            "{}/api/v1{}",
            self.config.base_url.trim_end_matches('/'),
            normalized
        )
    }

    async fn parse_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();
        let request_id = response
            .headers()
            .get("x-request-id")
            .and_then(|value| value.to_str().ok())
            .map(ToOwned::to_owned);
        if status.is_success() {
            let body = read_limited_response_body(response, MAX_RESPONSE_BODY_BYTES).await?;
            serde_json::from_slice::<T>(&body).map_err(|error| RainyError::Serialization {
                message: "failed to parse session response".to_string(),
                source_error: Some(error.to_string()),
            })
        } else {
            let body = read_limited_response_body(response, MAX_ERROR_BODY_BYTES).await?;
            let parsed = serde_json::from_slice::<ApiErrorResponse>(&body).ok();
            let access_token = self
                .access_token
                .as_ref()
                .map(|token| token.expose_secret());
            let (code, message) = parsed
                .map(|response| {
                    (
                        redact_secret(
                            &safe_error_component(&response.error.code, "API_ERROR"),
                            access_token,
                        ),
                        redact_secret(&safe_error_message(&response.error.message), access_token),
                    )
                })
                .unwrap_or_else(|| {
                    (
                        status
                            .canonical_reason()
                            .unwrap_or("HTTP_ERROR")
                            .to_string(),
                        "The service returned an invalid error response".to_string(),
                    )
                });
            Err(RainyError::Api {
                code,
                message,
                status_code: status.as_u16(),
                retryable: status.is_server_error(),
                request_id,
            })
        }
    }

    async fn request_json<T: DeserializeOwned, B: Serialize>(
        &self,
        method: Method,
        path: &str,
        body: Option<&B>,
        auth: bool,
    ) -> Result<T> {
        let mut request = self.client.request(method, self.api_v1_url(path));

        if auth {
            let token = self
                .access_token
                .as_ref()
                .ok_or_else(|| RainyError::Authentication {
                    code: "MISSING_SESSION_TOKEN".to_string(),
                    message: "Session access token is required for this operation".to_string(),
                    retryable: false,
                })?;
            let value = HeaderValue::from_str(&format!("Bearer {}", token.expose_secret()))
                .map_err(|_| RainyError::InvalidRequest {
                    code: "INVALID_SESSION_TOKEN".to_string(),
                    message: "session token cannot be used in an HTTP header".to_string(),
                    details: None,
                })?;
            request = request.header(AUTHORIZATION, value);
        }

        if let Some(body) = body {
            request = request
                .header(CONTENT_TYPE, "application/json")
                .body(serialize_json_body(body, GENERAL_REQUEST_BODY_BYTES)?);
        }

        let response = request.send().await.map_err(|error| {
            if error.is_timeout() {
                RainyError::Timeout {
                    message: "Request timed out".to_string(),
                    duration_ms: self.config.timeout_seconds.saturating_mul(1000),
                }
            } else {
                RainyError::Network {
                    message: if error.is_connect() {
                        "Could not connect to the service".to_string()
                    } else {
                        "The HTTP request failed".to_string()
                    },
                    retryable: error.is_connect() || error.is_request(),
                    source_error: None,
                }
            }
        })?;
        self.parse_response(response).await
    }

    /// Authenticates a user and stores the returned access token in the client.
    pub async fn login(&mut self, email: &str, password: &str) -> Result<LoginResponse> {
        let response: LoginResponse = self
            .request_json(
                Method::POST,
                "/auth/login",
                Some(&LoginRequest { email, password }),
                false,
            )
            .await?;
        self.set_access_token(response.access_token.clone());
        Ok(response)
    }

    /// Registers a user and stores the returned access token.
    pub async fn register(
        &mut self,
        email: &str,
        password: &str,
        region: &str,
    ) -> Result<LoginResponse> {
        let response: LoginResponse = self
            .request_json(
                Method::POST,
                "/auth/register",
                Some(&RegisterRequest {
                    email,
                    password,
                    region,
                }),
                false,
            )
            .await?;
        self.set_access_token(response.access_token.clone());
        Ok(response)
    }

    /// Refreshes the session token pair and stores the new access token.
    pub async fn refresh(&mut self, refresh_token: &str) -> Result<RefreshResponse> {
        let response: RefreshResponse = self
            .request_json(
                Method::POST,
                "/auth/refresh",
                Some(&RefreshRequest { refresh_token }),
                false,
            )
            .await?;
        self.set_access_token(response.access_token.clone());
        Ok(response)
    }

    /// Returns the current authenticated user profile from `GET /api/v1/auth/me`.
    pub async fn me(&self) -> Result<SessionUser> {
        let envelope: ApiEnvelope<SessionUser> = self
            .request_json::<ApiEnvelope<SessionUser>, serde_json::Value>(
                Method::GET,
                "/auth/me",
                None,
                true,
            )
            .await?;
        let _ = envelope.success;
        Ok(envelope.data)
    }

    /// Returns the current organization profile from `GET /api/v1/orgs/me`.
    pub async fn org_me(&self) -> Result<OrgProfile> {
        self.request_json(
            Method::GET,
            "/orgs/me",
            Option::<&serde_json::Value>::None,
            true,
        )
        .await
    }

    /// Lists API keys for the authenticated organization/user session.
    pub async fn list_api_keys(&self) -> Result<Vec<SessionApiKeyListItem>> {
        let response: ListKeysEnvelope = self
            .request_json(
                Method::GET,
                "/keys",
                Option::<&serde_json::Value>::None,
                true,
            )
            .await?;
        let _ = response.success;
        Ok(response.keys)
    }

    /// Creates a new API key for the authenticated session.
    pub async fn create_api_key(
        &self,
        name: &str,
        key_type: Option<&str>,
    ) -> Result<CreatedApiKey> {
        #[derive(Serialize)]
        struct CreateKeyRequest<'a> {
            name: &'a str,
            #[serde(skip_serializing_if = "Option::is_none")]
            r#type: Option<&'a str>,
        }
        self.request_json(
            Method::POST,
            "/keys",
            Some(&CreateKeyRequest {
                name,
                r#type: key_type,
            }),
            true,
        )
        .await
    }

    /// Deletes an API key by ID.
    pub async fn delete_api_key(&self, id: &str) -> Result<serde_json::Value> {
        self.request_json(
            Method::DELETE,
            &format!("/keys/{id}"),
            Option::<&serde_json::Value>::None,
            true,
        )
        .await
    }

    /// Returns current credit balance information from `GET /api/v1/usage/credits`.
    pub async fn usage_credits(&self) -> Result<UsageCreditsResponse> {
        self.request_json(
            Method::GET,
            "/usage/credits",
            Option::<&serde_json::Value>::None,
            true,
        )
        .await
    }

    /// Returns usage statistics from `GET /api/v1/usage/stats`.
    pub async fn usage_stats(&self, days: Option<u32>) -> Result<UsageStatsResponse> {
        let path = match days {
            Some(days) => format!("/usage/stats?days={days}"),
            None => "/usage/stats".to_string(),
        };
        self.request_json(Method::GET, &path, Option::<&serde_json::Value>::None, true)
            .await
    }
}

impl fmt::Debug for RainySessionClient {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RainySessionClient")
            .field("base_url", &safe_url_label(&self.config.base_url))
            .field("timeout_seconds", &self.config.timeout_seconds)
            .field(
                "access_token",
                &self.access_token.as_ref().map(|_| "<redacted>"),
            )
            .finish()
    }
}

impl fmt::Debug for SessionTokens {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("SessionTokens")
            .field("access_token", &"<redacted>")
            .field("refresh_token", &"<redacted>")
            .finish()
    }
}

impl fmt::Debug for LoginResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("LoginResponse")
            .field("access_token", &"<redacted>")
            .field("refresh_token", &"<redacted>")
            .field("user", &self.user)
            .finish()
    }
}

impl fmt::Debug for RefreshResponse {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("RefreshResponse")
            .field("access_token", &"<redacted>")
            .field("refresh_token", &"<redacted>")
            .finish()
    }
}

impl fmt::Debug for CreatedApiKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("CreatedApiKey")
            .field("key", &"<redacted>")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("type", &self.r#type)
            .finish()
    }
}

fn safe_error_component(value: &str, fallback: &str) -> String {
    let mut output = value
        .chars()
        .filter(|character| !character.is_control())
        .collect::<String>();
    if output.is_empty() {
        fallback.to_string()
    } else {
        if output.chars().count() > 128 {
            output = output.chars().take(128).collect();
            output.push('…');
        }
        output
    }
}

fn safe_error_message(value: &str) -> String {
    let mut output = value
        .chars()
        .filter(|character| !character.is_control() || matches!(character, '\n' | '\t'))
        .collect::<String>();
    if output.chars().count() > 1024 {
        output = output.chars().take(1024).collect();
        output.push('…');
    }
    if output.is_empty() {
        "The service returned an error".to_string()
    } else {
        output
    }
}

fn redact_secret(value: &str, secret: Option<&str>) -> String {
    match secret.filter(|secret| !secret.is_empty()) {
        Some(secret) => value.replace(secret, "<redacted>"),
        None => value.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_client_uses_v3_base_url() {
        let client = RainySessionClient::new().expect("session client");
        assert!(client.base_url().starts_with("https://"));
        assert_eq!(
            client.api_v1_url("/auth/login"),
            format!("{}/api/v1/auth/login", client.base_url())
        );
    }

    #[test]
    fn parses_login_alias_shape() {
        let payload = r#"{
          "success": true,
          "data": {"accessToken":"access-token-secret","refreshToken":"refresh-token-secret","user":{"id":"1","email":"e@x.com","role":"admin"}},
          "accessToken":"access-token-secret",
          "refreshToken":"refresh-token-secret",
          "user":{"id":"1","email":"e@x.com","role":"admin"}
        }"#;
        let parsed: LoginResponse = serde_json::from_str(payload).expect("deserialize login");
        assert_eq!(parsed.access_token, "access-token-secret");
        assert_eq!(parsed.refresh_token, "refresh-token-secret");
        assert_eq!(parsed.user.email, "e@x.com");
        assert!(!format!("{parsed:?}").contains("access-token-secret"));
        assert!(!format!("{parsed:?}").contains("refresh-token-secret"));
    }

    #[test]
    fn session_client_rejects_non_loopback_http() {
        let result = RainySessionClient::with_base_url("http://example.com");
        assert!(result.is_err());
    }

    #[test]
    fn session_config_debug_uses_a_safe_url_label() {
        let config = SessionConfig::new()
            .with_base_url("https://user:password@example.com/private?token=should-not-print");
        let rendered = format!("{config:?}");
        assert!(!rendered.contains("user:password"));
        assert!(!rendered.contains("token=should-not-print"));
        assert!(rendered.contains("https://example.com"));
    }
}
