//! JWT/session client for Rainy API v3 dashboard-style endpoints.
//!
//! This module intentionally separates session/JWT operations from `RainyClient` (API-key flows)
//! to keep trust boundaries clear and the default SDK surface smaller.

use crate::error::{ApiErrorResponse, RainyError, Result};
use reqwest::{
    header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE, USER_AGENT},
    Client, Method, Response,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// Configuration for [`RainySessionClient`].
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Base URL of the Rainy API v3 service (host only; API paths are added by the client).
    pub base_url: String,
    /// HTTP timeout in seconds for session requests.
    pub timeout_seconds: u64,
    /// User-Agent header used for session requests.
    pub user_agent: String,
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
/// Use this client for authentication and dashboard/account operations such as
/// `/api/v1/auth/*`, `/api/v1/keys`, `/api/v1/usage/*`, and `/api/v1/orgs/me`.
#[derive(Debug, Clone)]
pub struct RainySessionClient {
    client: Client,
    config: SessionConfig,
    access_token: Option<String>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionTokens {
    /// Access token for authenticated session requests.
    #[serde(rename = "accessToken")]
    pub access_token: String,
    /// Refresh token used to renew the access token.
    #[serde(rename = "refreshToken")]
    pub refresh_token: String,
}

/// Response payload for login/register auth endpoints.
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Provider-level summary alias payload.
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
        if url::Url::parse(&config.base_url).is_err() {
            return Err(RainyError::InvalidRequest {
                code: "INVALID_BASE_URL".to_string(),
                message: "Base URL is not a valid URL".to_string(),
                details: None,
            });
        }

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        headers.insert(
            USER_AGENT,
            HeaderValue::from_str(&config.user_agent).map_err(|e| RainyError::Network {
                message: format!("Invalid user agent: {e}"),
                retryable: false,
                source_error: Some(e.to_string()),
            })?,
        );

        let client = Client::builder()
            .use_rustls_tls()
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .default_headers(headers)
            .build()
            .map_err(|e| RainyError::Network {
                message: format!("Failed to create HTTP client: {e}"),
                retryable: false,
                source_error: Some(e.to_string()),
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
        self.access_token = Some(access_token.into());
    }

    /// Clears the in-memory access token.
    pub fn clear_access_token(&mut self) {
        self.access_token = None;
    }

    /// Returns the current in-memory access token, if set.
    pub fn access_token(&self) -> Option<&str> {
        self.access_token.as_deref()
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
            .and_then(|v| v.to_str().ok())
            .map(ToOwned::to_owned);
        let text = response.text().await.unwrap_or_default();

        if status.is_success() {
            serde_json::from_str::<T>(&text).map_err(|e| RainyError::Serialization {
                message: format!("Failed to parse response: {e}"),
                source_error: Some(e.to_string()),
            })
        } else if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&text) {
            let error = error_response.error;
            Err(RainyError::Api {
                code: error.code,
                message: error.message,
                status_code: status.as_u16(),
                retryable: status.is_server_error(),
                request_id,
            })
        } else {
            Err(RainyError::Api {
                code: status.canonical_reason().unwrap_or("UNKNOWN").to_string(),
                message: if text.is_empty() {
                    format!("HTTP {}", status.as_u16())
                } else {
                    text
                },
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
            request = request.header(AUTHORIZATION, format!("Bearer {token}"));
        }

        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await?;
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
        self.access_token = Some(response.access_token.clone());
        Ok(response)
    }

    /// Registers a user and stores the returned access token in the client.
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
        self.access_token = Some(response.access_token.clone());
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
        self.access_token = Some(response.access_token.clone());
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
        let response: OrgProfile = self
            .request_json(
                Method::GET,
                "/orgs/me",
                Option::<&serde_json::Value>::None,
                true,
            )
            .await?;
        Ok(response)
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
    ///
    /// `key_type` may be `Some("standard")`, `Some("platform")`, or `None`
    /// to let the server default apply.
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
        let response: CreatedApiKey = self
            .request_json(
                Method::POST,
                "/keys",
                Some(&CreateKeyRequest {
                    name,
                    r#type: key_type,
                }),
                true,
            )
            .await?;
        Ok(response)
    }

    /// Deletes an API key by ID.
    ///
    /// Returns the server JSON response as-is to avoid over-expanding the SDK surface.
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
    ///
    /// When `days` is `None`, the server default period is used.
    pub async fn usage_stats(&self, days: Option<u32>) -> Result<UsageStatsResponse> {
        let path = match days {
            Some(days) => format!("/usage/stats?days={days}"),
            None => "/usage/stats".to_string(),
        };
        self.request_json(Method::GET, &path, Option::<&serde_json::Value>::None, true)
            .await
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
          "data": {"accessToken":"a","refreshToken":"r","user":{"id":"1","email":"e@x.com","role":"admin"}},
          "accessToken":"a",
          "refreshToken":"r",
          "user":{"id":"1","email":"e@x.com","role":"admin"}
        }"#;
        let parsed: LoginResponse = serde_json::from_str(payload).expect("deserialize login");
        assert_eq!(parsed.access_token, "a");
        assert_eq!(parsed.refresh_token, "r");
        assert_eq!(parsed.user.email, "e@x.com");
    }
}
