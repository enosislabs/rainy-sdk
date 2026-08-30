//! JWT/session client for Rainy API v3 dashboard-style endpoints.
//!
//! This module intentionally separates session/JWT operations from `RainyClient` (API-key flows)
//! to keep trust boundaries clear and the default SDK surface smaller.

use crate::{
    auth::{
        GENERAL_REQUEST_BODY_BYTES, MAX_ERROR_BODY_BYTES, MAX_RESPONSE_BODY_BYTES,
        read_limited_response_body, retry_after_seconds, serialize_json_body, validate_service_url,
    },
    error::{ApiErrorResponse, RainyError, Result},
    models::{RegisteredTool, Tool, ToolDefinition, ToolUpdate},
};
use reqwest::{
    Client, Method, Response,
    header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue, USER_AGENT},
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::Value;
use std::collections::HashMap;

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
#[derive(Clone)]
pub struct RainySessionClient {
    client: Client,
    config: SessionConfig,
    access_token: Option<String>,
}

/// Request body for `POST /api/v1/auth/login`.
#[derive(Clone, Serialize)]
pub struct LoginRequest<'a> {
    /// User email address.
    pub email: &'a str,
    /// User password.
    pub password: &'a str,
}

/// Request body for `POST /api/v1/auth/register`.
#[derive(Clone, Serialize)]
pub struct RegisterRequest<'a> {
    /// User email address.
    pub email: &'a str,
    /// User password.
    pub password: &'a str,
    /// Region code (for example `us` or `la`).
    pub region: &'a str,
}

/// Request body for `POST /api/v1/auth/register-with-invite`.
#[derive(Clone, Serialize)]
pub struct RegisterWithInviteRequest<'a> {
    /// Invitation token issued by the organization.
    pub token: &'a str,
    /// Email address associated with the invitation.
    pub email: &'a str,
    /// Password for the new account.
    pub password: &'a str,
}

/// Request body for `POST /api/v1/auth/refresh`.
#[derive(Clone, Serialize)]
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
    /// Better Auth's identity identifier, when included.
    #[serde(rename = "betterAuthUserId", default)]
    pub better_auth_user_id: Option<String>,
    /// Alias for the Better Auth identity identifier.
    #[serde(rename = "authUserId", default)]
    pub auth_user_id: Option<String>,
    /// Organization membership identifier, when included.
    #[serde(rename = "membershipId", default)]
    pub membership_id: Option<String>,
    /// Selected organization identifier, when returned by `/auth/me`.
    #[serde(rename = "selectedOrgId", default)]
    pub selected_org_id: Option<String>,
    /// Session capabilities returned by the API.
    #[serde(default)]
    pub capabilities: Option<Value>,
    /// Forward-compatible user fields.
    #[serde(flatten, default)]
    pub extra: HashMap<String, Value>,
}

/// One organization membership returned by login or `/auth/me`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMembership {
    /// Membership identifier.
    #[serde(rename = "membershipId")]
    pub membership_id: String,
    /// Organization identifier.
    #[serde(rename = "orgId")]
    pub org_id: String,
    /// Membership role.
    pub role: String,
    /// Membership status.
    #[serde(default)]
    pub status: Option<String>,
    /// Forward-compatible membership fields.
    #[serde(flatten, default)]
    pub extra: HashMap<String, Value>,
}

/// Organization user summary returned by the administrator user list.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUserSummary {
    /// Membership or Better Auth user identifier.
    pub id: String,
    /// Legacy user identifier, when the account has one.
    #[serde(rename = "legacyUserId", default)]
    pub legacy_user_id: Option<String>,
    /// User email address.
    pub email: String,
    /// Current organization role.
    pub role: String,
    /// Last activity timestamp, when available.
    #[serde(rename = "lastActiveAt", default)]
    pub last_active_at: Option<String>,
    /// Membership creation timestamp.
    #[serde(rename = "createdAt")]
    pub created_at: String,
    /// Forward-compatible user fields.
    #[serde(flatten, default)]
    pub extra: HashMap<String, Value>,
}

/// Request body for inviting a user through the legacy email invitation route.
#[derive(Debug, Clone, Serialize)]
pub struct UserInviteRequest<'a> {
    /// Email address to invite.
    pub email: &'a str,
    /// Organization role (`admin` or `member`).
    pub role: &'a str,
}

/// Request body for changing an organization's user role.
#[derive(Debug, Clone, Serialize)]
pub struct UserRoleRequest<'a> {
    /// New organization role (`admin` or `member`).
    pub role: &'a str,
}

/// Request body for purchasing a credit package.
#[derive(Debug, Clone, Serialize)]
pub struct CheckoutRequest<'a> {
    /// Credit package identifier.
    #[serde(rename = "packageId")]
    pub package_id: &'a str,
    /// Optional package quantity.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<u32>,
    /// Optional allow-listed return URL supplied to the billing provider.
    #[serde(rename = "successUrl", skip_serializing_if = "Option::is_none")]
    pub success_url: Option<&'a str>,
    /// Optional allow-listed cancellation URL.
    #[serde(rename = "cancelUrl", skip_serializing_if = "Option::is_none")]
    pub cancel_url: Option<&'a str>,
    /// Optional allow-listed return URL.
    #[serde(rename = "returnUrl", skip_serializing_if = "Option::is_none")]
    pub return_url: Option<&'a str>,
}

/// Request body for purchasing a paid subscription plan.
#[derive(Debug, Clone, Serialize)]
pub struct PlanCheckoutRequest<'a> {
    /// Public subscription plan code.
    #[serde(rename = "planCode")]
    pub plan_code: &'a str,
    /// Optional allow-listed success URL.
    #[serde(rename = "successUrl", skip_serializing_if = "Option::is_none")]
    pub success_url: Option<&'a str>,
    /// Optional allow-listed cancellation URL.
    #[serde(rename = "cancelUrl", skip_serializing_if = "Option::is_none")]
    pub cancel_url: Option<&'a str>,
    /// Optional allow-listed return URL.
    #[serde(rename = "returnUrl", skip_serializing_if = "Option::is_none")]
    pub return_url: Option<&'a str>,
}

/// Request body for subscription plan changes.
#[derive(Debug, Clone, Serialize)]
pub struct PlanChangeRequest<'a> {
    /// Target public plan code.
    #[serde(rename = "planCode")]
    pub plan_code: &'a str,
    /// Optional allow-listed success URL.
    #[serde(rename = "successUrl", skip_serializing_if = "Option::is_none")]
    pub success_url: Option<&'a str>,
    /// Optional allow-listed cancellation URL.
    #[serde(rename = "cancelUrl", skip_serializing_if = "Option::is_none")]
    pub cancel_url: Option<&'a str>,
    /// Optional allow-listed return URL.
    #[serde(rename = "returnUrl", skip_serializing_if = "Option::is_none")]
    pub return_url: Option<&'a str>,
}

/// Request body for opening a billing portal session.
#[derive(Debug, Clone, Serialize)]
pub struct PortalSessionRequest<'a> {
    /// Optional allow-listed return URL.
    #[serde(rename = "returnUrl", skip_serializing_if = "Option::is_none")]
    pub return_url: Option<&'a str>,
}

/// Common typed response fields returned by billing actions and checkout
/// status. Unknown provider fields remain in `extra`.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BillingActionResponse {
    /// Whether a matching checkout session was found.
    #[serde(default)]
    pub found: Option<bool>,
    /// Legacy or internal checkout state.
    #[serde(default)]
    pub status: Option<String>,
    /// Provider checkout URL.
    #[serde(rename = "checkoutUrl", default)]
    pub checkout_url: Option<String>,
    /// Generic URL alias.
    #[serde(default)]
    pub url: Option<String>,
    /// Provider session identifier.
    #[serde(rename = "sessionId", default)]
    pub session_id: Option<String>,
    /// Internal checkout-intent identifier.
    #[serde(rename = "intentId", default)]
    pub intent_id: Option<String>,
    /// Whether an existing intent was reused.
    #[serde(default)]
    pub reused: Option<bool>,
    /// Credit package identifier.
    #[serde(rename = "packageId", default)]
    pub package_id: Option<String>,
    /// Checkout package type.
    #[serde(rename = "packageType", default)]
    pub package_type: Option<String>,
    /// Subscription plan code.
    #[serde(rename = "planCode", default)]
    pub plan_code: Option<String>,
    /// Target plan code for a plan-change action.
    #[serde(rename = "targetPlanCode", default)]
    pub target_plan_code: Option<String>,
    /// Internal provider status.
    #[serde(rename = "internalStatus", default)]
    pub internal_status: Option<String>,
    /// Provider order identifier.
    #[serde(rename = "orderId", default)]
    pub order_id: Option<String>,
    /// Provider status value.
    #[serde(rename = "providerStatus", default)]
    pub provider_status: Option<String>,
    /// Human-readable terminal reason.
    #[serde(rename = "terminalReason", default)]
    pub terminal_reason: Option<String>,
    /// Provider/session expiry timestamp.
    #[serde(rename = "expiresAt", default)]
    pub expires_at: Option<String>,
    /// Last status update timestamp.
    #[serde(rename = "updatedAt", default)]
    pub updated_at: Option<String>,
    /// Billing portal URL.
    #[serde(rename = "portalUrl", default)]
    pub portal_url: Option<String>,
    /// Explicit API success marker when top-level aliases are returned.
    #[serde(default)]
    pub success: Option<bool>,
    /// Nested response data when the server does not emit aliases.
    #[serde(default)]
    pub data: Option<Value>,
    /// Provider/package fields added after this SDK release.
    #[serde(flatten, default)]
    pub extra: HashMap<String, Value>,
}

/// Pair of access and refresh tokens.
#[derive(Clone, Serialize, Deserialize)]
pub struct SessionTokens {
    /// Access token for authenticated session requests.
    #[serde(rename = "accessToken")]
    pub access_token: String,
    /// Refresh token used to renew the access token.
    #[serde(rename = "refreshToken")]
    pub refresh_token: Option<String>,
}

/// Response payload for login/register auth endpoints.
#[derive(Clone, Serialize, Deserialize)]
pub struct LoginResponse {
    /// Access token returned by the API.
    #[serde(rename = "accessToken")]
    pub access_token: String,
    /// Refresh token returned by the API.
    #[serde(rename = "refreshToken")]
    pub refresh_token: Option<String>,
    /// Authenticated user information.
    pub user: SessionUser,
    /// Organizations available to the authenticated identity.
    #[serde(default)]
    pub memberships: Vec<SessionMembership>,
    /// Organization selected by the API for this session.
    #[serde(rename = "selectedOrgId", default)]
    pub selected_org_id: Option<String>,
    /// Forward-compatible auth response fields.
    #[serde(flatten, default)]
    pub extra: HashMap<String, Value>,
}

/// Response payload for token refresh.
#[derive(Clone, Serialize, Deserialize)]
pub struct RefreshResponse {
    /// New access token.
    #[serde(rename = "accessToken")]
    pub access_token: String,
    /// Rotated or reissued refresh token.
    #[serde(rename = "refreshToken")]
    pub refresh_token: Option<String>,
    /// Forward-compatible response fields.
    #[serde(flatten, default)]
    pub extra: HashMap<String, Value>,
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

/// Result returned by the unauthenticated dual-key validation endpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyValidationResponse {
    /// Organization associated with the key pair.
    #[serde(rename = "orgId")]
    pub org_id: String,
    /// Standard API-key identifier associated with the key pair.
    #[serde(rename = "apiKeyId")]
    pub api_key_id: String,
}

/// One organization invitation returned by the session API.
#[derive(Clone, Serialize, Deserialize)]
pub struct SessionInvitation {
    /// Invitation identifier.
    pub id: String,
    /// Invitation token. Treat this as a credential.
    pub token: String,
    /// Assigned organization role.
    pub role: String,
    /// ISO-8601 expiration timestamp.
    #[serde(rename = "expiresAt")]
    pub expires_at: String,
    /// ISO-8601 creation timestamp.
    #[serde(rename = "createdAt")]
    pub created_at: String,
}

/// Organization privacy and telemetry settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgSettings {
    /// Privacy mode selected by the organization.
    #[serde(rename = "privacyMode")]
    pub privacy_mode: String,
    /// Whether customer telemetry is enabled.
    #[serde(rename = "telemetryEnabled")]
    pub telemetry_enabled: bool,
}

/// One model access record returned by `GET /api/v1/orgs/me/models`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrgModelAccess {
    /// Model identifier.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Whether the organization has enabled the model.
    pub enabled: bool,
    /// Whether the current plan allows the model.
    #[serde(rename = "planAllowed")]
    pub plan_allowed: bool,
    /// Whether the model satisfies the active privacy mode.
    #[serde(rename = "privacyCompatible")]
    pub privacy_compatible: bool,
    /// Derived capability metadata.
    #[serde(default)]
    pub capabilities: Option<Value>,
    /// Public data-policy metadata.
    #[serde(rename = "dataPolicy", default)]
    pub data_policy: Option<Value>,
    /// Product tier.
    #[serde(default)]
    pub tier: Option<String>,
    /// Forward-compatible model fields.
    #[serde(flatten, default)]
    pub extra: HashMap<String, Value>,
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
    /// Credit breakdown by source/category.
    #[serde(default)]
    pub breakdown: Option<Value>,
    /// Effective billing plan code.
    #[serde(default)]
    pub plan: Option<String>,
    /// Subscription status, when available.
    #[serde(rename = "subscriptionStatus", default)]
    pub subscription_status: Option<String>,
    /// Current subscription period end.
    #[serde(rename = "currentPeriodEnd", default)]
    pub current_period_end: Option<String>,
    /// Auto-recharge policy and current setting.
    #[serde(rename = "autoRecharge", default)]
    pub auto_recharge: Option<Value>,
}

/// Usage statistics response for `GET /api/v1/usage/stats`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStatsResponse {
    /// Number of days included in the aggregation window.
    #[serde(rename = "periodDays")]
    #[serde(default)]
    pub period_days: u32,
    /// Total requests in the selected period.
    #[serde(rename = "totalRequests")]
    #[serde(default)]
    pub total_requests: u64,
    /// Total credits deducted in the selected period.
    #[serde(rename = "totalCreditsDeducted")]
    #[serde(default)]
    pub total_credits_deducted: f64,
    /// Total prompt tokens in the selected period.
    #[serde(rename = "totalPromptTokens", default)]
    pub total_prompt_tokens: u64,
    /// Total completion tokens in the selected period.
    #[serde(rename = "totalCompletionTokens", default)]
    pub total_completion_tokens: u64,
    /// Total tokens in the selected period.
    #[serde(rename = "totalTokens", default)]
    pub total_tokens: u64,
    /// Number of non-unknown providers used.
    #[serde(rename = "providersUsed", default)]
    pub providers_used: u64,
    /// Provider/model aggregates.
    #[serde(rename = "byProvider", default)]
    pub by_provider: Value,
    /// Model aggregates.
    #[serde(rename = "byModel", default)]
    pub by_model: Value,
    /// Daily credit limit snapshot.
    #[serde(default)]
    pub daily: Value,
    /// Provider-level summary alias payload.
    #[serde(rename = "statsByProvider", default)]
    pub stats_by_provider: serde_json::Value,
    /// Model aggregate alias payload.
    #[serde(rename = "statsByModel", default)]
    pub stats_by_model: Value,
    /// Model-tier aggregate alias payload.
    #[serde(rename = "statsByModelTier", default)]
    pub stats_by_model_tier: Value,
    /// API-key aggregate alias payload.
    #[serde(rename = "statsByApiKey", default)]
    pub stats_by_api_key: Value,
    /// Daily aggregate rows.
    #[serde(rename = "dailyUsage", default)]
    pub daily_usage: Vec<Value>,
    /// Recent usage rows.
    #[serde(rename = "recentLogs", default)]
    pub recent_logs: Vec<Value>,
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

fn encode_path_segment(value: &str) -> String {
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~') {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(char::from(b"0123456789ABCDEF"[(byte >> 4) as usize]));
            encoded.push(char::from(b"0123456789ABCDEF"[(byte & 0x0f) as usize]));
        }
    }
    encoded
}

impl std::fmt::Debug for RainySessionClient {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RainySessionClient")
            .field("base_url", &self.config.base_url)
            .field("timeout_seconds", &self.config.timeout_seconds)
            .field("access_token", &"[REDACTED]")
            .finish()
    }
}

impl std::fmt::Debug for LoginRequest<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LoginRequest")
            .field("email", &self.email)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

impl std::fmt::Debug for RegisterRequest<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RegisterRequest")
            .field("email", &self.email)
            .field("password", &"[REDACTED]")
            .field("region", &self.region)
            .finish()
    }
}

impl std::fmt::Debug for RegisterWithInviteRequest<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RegisterWithInviteRequest")
            .field("token", &"[REDACTED]")
            .field("email", &self.email)
            .field("password", &"[REDACTED]")
            .finish()
    }
}

impl std::fmt::Debug for RefreshRequest<'_> {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RefreshRequest")
            .field("refresh_token", &"[REDACTED]")
            .finish()
    }
}

impl std::fmt::Debug for SessionTokens {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SessionTokens")
            .field("access_token", &"[REDACTED]")
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[REDACTED]"),
            )
            .finish()
    }
}

impl std::fmt::Debug for LoginResponse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("LoginResponse")
            .field("access_token", &"[REDACTED]")
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[REDACTED]"),
            )
            .field("user", &self.user)
            .field("memberships", &self.memberships)
            .field("selected_org_id", &self.selected_org_id)
            .finish()
    }
}

impl std::fmt::Debug for RefreshResponse {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("RefreshResponse")
            .field("access_token", &"[REDACTED]")
            .field(
                "refresh_token",
                &self.refresh_token.as_ref().map(|_| "[REDACTED]"),
            )
            .finish()
    }
}

impl std::fmt::Debug for CreatedApiKey {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("CreatedApiKey")
            .field("key", &"[REDACTED]")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("type", &self.r#type)
            .finish()
    }
}

impl std::fmt::Debug for SessionInvitation {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SessionInvitation")
            .field("id", &self.id)
            .field("token", &"[REDACTED]")
            .field("role", &self.role)
            .field("expires_at", &self.expires_at)
            .field("created_at", &self.created_at)
            .finish()
    }
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
                message: "Session timeout must be greater than zero".to_string(),
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
            .tls_backend_rustls()
            .min_tls_version(reqwest::tls::Version::TLS_1_2)
            .redirect(reqwest::redirect::Policy::none())
            .timeout(std::time::Duration::from_secs(config.timeout_seconds))
            .default_headers(headers)
            .build()
            .map_err(|e| RainyError::Network {
                message: format!("Failed to create HTTP client: {e}"),
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
        let retry_after = retry_after_seconds(response.headers());
        if status.is_success() {
            let body = read_limited_response_body(response, MAX_RESPONSE_BODY_BYTES).await?;
            serde_json::from_slice::<T>(&body).map_err(|e| RainyError::Serialization {
                message: format!("Failed to parse response: {e}"),
                source_error: Some(e.to_string()),
            })
        } else {
            let text = read_limited_response_body(response, MAX_ERROR_BODY_BYTES)
                .await
                .map(|body| String::from_utf8_lossy(&body).into_owned())?;
            if let Ok(error_response) = serde_json::from_str::<ApiErrorResponse>(&text) {
                let error = error_response.error;
                let retryable = error.retryable.unwrap_or(status.is_server_error());
                let code = error.code;
                let message = error.message;
                let details = error.details;
                match (status, code.as_str()) {
                    (reqwest::StatusCode::UNAUTHORIZED, _)
                    | (_, "UNAUTHORIZED" | "INVALID_TOKEN" | "SESSION_EXPIRED") => {
                        Err(RainyError::Authentication {
                            code,
                            message,
                            retryable: false,
                        })
                    }
                    (reqwest::StatusCode::FORBIDDEN, _)
                    | (_, "FORBIDDEN" | "ACCESS_DENIED" | "ORG_ADMIN_REQUIRED") => {
                        Err(RainyError::AccessDenied {
                            code,
                            message,
                            details,
                        })
                    }
                    (reqwest::StatusCode::GONE, _) | (_, "REFRESH_UNSUPPORTED") => {
                        Err(RainyError::FeatureNotAvailable {
                            feature: "session_refresh".to_string(),
                            message,
                        })
                    }
                    (reqwest::StatusCode::TOO_MANY_REQUESTS, _) => Err(RainyError::RateLimit {
                        code,
                        message,
                        retry_after: details
                            .as_ref()
                            .and_then(|value| value.get("retry_after"))
                            .and_then(Value::as_u64)
                            .or(retry_after),
                        current_usage: None,
                    }),
                    _ => Err(RainyError::Api {
                        code,
                        message,
                        status_code: status.as_u16(),
                        retryable,
                        request_id,
                    }),
                }
            } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                Err(RainyError::RateLimit {
                    code: "RATE_LIMIT_EXCEEDED".to_string(),
                    message: if text.is_empty() {
                        "Too many requests".to_string()
                    } else {
                        text
                    },
                    retry_after,
                    current_usage: None,
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
            let body = serialize_json_body(body, GENERAL_REQUEST_BODY_BYTES)?;
            request = request.header(CONTENT_TYPE, "application/json").body(body);
        }

        let response = request.send().await.map_err(|e| {
            if e.is_timeout() {
                RainyError::Timeout {
                    message: "Session request timed out".to_string(),
                    duration_ms: self.config.timeout_seconds.saturating_mul(1000),
                }
            } else {
                RainyError::Network {
                    message: "Failed to send session request".to_string(),
                    retryable: false,
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

    /// Registers an account from an organization invitation and stores the
    /// returned access token.
    pub async fn register_with_invite(
        &mut self,
        token: &str,
        email: &str,
        password: &str,
    ) -> Result<LoginResponse> {
        let response: LoginResponse = self
            .request_json(
                Method::POST,
                "/auth/register-with-invite",
                Some(&RegisterWithInviteRequest {
                    token,
                    email,
                    password,
                }),
                false,
            )
            .await?;
        self.access_token = Some(response.access_token.clone());
        Ok(response)
    }

    /// Refreshes the session token pair and stores the new access token.
    pub async fn refresh(&mut self, refresh_token: &str) -> Result<RefreshResponse> {
        let _ = refresh_token;
        Err(RainyError::FeatureNotAvailable {
            feature: "session_refresh".to_string(),
            message: "The current Rainy auth service intentionally disables refresh-token rotation; authenticate again with login or register".to_string(),
        })
    }

    /// Invalidates the current Better Auth session, when one is set.
    pub async fn logout(&mut self) -> Result<()> {
        let authenticated = self.access_token.is_some();
        let _: Value = self
            .request_json(
                Method::POST,
                "/auth/logout",
                Option::<&Value>::None,
                authenticated,
            )
            .await?;
        self.clear_access_token();
        Ok(())
    }

    /// Deletes the current account and clears the in-memory session token.
    pub async fn delete_account(&mut self) -> Result<()> {
        let _: Value = self
            .request_json(
                Method::DELETE,
                "/auth/account",
                Option::<&Value>::None,
                true,
            )
            .await?;
        self.clear_access_token();
        Ok(())
    }

    /// Lists active users in the current organization. The API requires an
    /// administrator session for this operation.
    pub async fn list_users(&self) -> Result<Vec<SessionUserSummary>> {
        let envelope: ApiEnvelope<Vec<SessionUserSummary>> = self
            .request_json(Method::GET, "/users", Option::<&Value>::None, true)
            .await?;
        Ok(envelope.data)
    }

    /// Invites a user by email through the compatibility invitation route.
    /// Prefer [`Self::create_invitation`] when no email-specific invitation is
    /// needed.
    pub async fn invite_user(&self, email: &str, role: &str) -> Result<SessionInvitation> {
        #[derive(Deserialize)]
        struct InviteData {
            invitation: SessionInvitation,
        }
        let envelope: ApiEnvelope<InviteData> = self
            .request_json(
                Method::POST,
                "/users/invite",
                Some(&UserInviteRequest { email, role }),
                true,
            )
            .await?;
        Ok(envelope.data.invitation)
    }

    /// Changes the organization role for one user. The API requires an
    /// administrator session.
    pub async fn update_user_role(&self, user_id: &str, role: &str) -> Result<Value> {
        let path = format!("/users/{}/role", encode_path_segment(user_id));
        let envelope: ApiEnvelope<Value> = self
            .request_json(Method::PATCH, &path, Some(&UserRoleRequest { role }), true)
            .await?;
        Ok(envelope.data)
    }

    /// Removes one user from the current organization. The API requires an
    /// administrator session and refuses self-removal.
    pub async fn remove_user(&self, user_id: &str) -> Result<Value> {
        let path = format!("/users/{}", encode_path_segment(user_id));
        let envelope: ApiEnvelope<Value> = self
            .request_json(Method::DELETE, &path, Option::<&Value>::None, true)
            .await?;
        Ok(envelope.data)
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

    /// Lists pending organization invitations. The API requires an admin
    /// session for this operation.
    pub async fn list_invitations(&self) -> Result<Vec<SessionInvitation>> {
        let envelope: ApiEnvelope<Vec<SessionInvitation>> = self
            .request_json(
                Method::GET,
                "/users/invitations",
                Option::<&Value>::None,
                true,
            )
            .await?;
        Ok(envelope.data)
    }

    /// Creates an organization invitation with the requested role.
    pub async fn create_invitation(&self, role: &str) -> Result<SessionInvitation> {
        #[derive(Serialize)]
        struct CreateInvitationRequest<'a> {
            role: &'a str,
        }
        #[derive(Deserialize)]
        struct CreateInvitationData {
            invitation: SessionInvitation,
        }
        let envelope: ApiEnvelope<CreateInvitationData> = self
            .request_json(
                Method::POST,
                "/users/invitations",
                Some(&CreateInvitationRequest { role }),
                true,
            )
            .await?;
        Ok(envelope.data.invitation)
    }

    /// Revokes a pending organization invitation.
    pub async fn revoke_invitation(&self, invitation_id: &str) -> Result<()> {
        let _: Value = self
            .request_json(
                Method::DELETE,
                &format!("/users/invitations/{}", encode_path_segment(invitation_id)),
                Option::<&Value>::None,
                true,
            )
            .await?;
        Ok(())
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

    /// Returns organization privacy and telemetry settings.
    pub async fn org_settings(&self) -> Result<OrgSettings> {
        let envelope: ApiEnvelope<OrgSettings> = self
            .request_json(
                Method::GET,
                "/orgs/me/settings",
                Option::<&Value>::None,
                true,
            )
            .await?;
        Ok(envelope.data)
    }

    /// Updates the organization's region metadata.
    pub async fn update_org_region(&self, region: &str) -> Result<()> {
        #[derive(Serialize)]
        struct RegionRequest<'a> {
            region: &'a str,
        }
        let _: Value = self
            .request_json(
                Method::PATCH,
                "/orgs/me/region",
                Some(&RegionRequest { region }),
                true,
            )
            .await?;
        Ok(())
    }

    /// Enables or disables organization telemetry.
    pub async fn set_telemetry_enabled(&self, enabled: bool) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct TelemetryRequest {
            telemetry_enabled: bool,
        }
        let _: Value = self
            .request_json(
                Method::PATCH,
                "/orgs/me/settings/telemetry",
                Some(&TelemetryRequest {
                    telemetry_enabled: enabled,
                }),
                true,
            )
            .await?;
        Ok(())
    }

    /// Sets the organization privacy mode (`standard`, `no_training`, or `zdr`).
    pub async fn set_privacy_mode(&self, privacy_mode: &str) -> Result<()> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct PrivacyRequest<'a> {
            privacy_mode: &'a str,
        }
        let _: Value = self
            .request_json(
                Method::PATCH,
                "/orgs/me/settings/privacy",
                Some(&PrivacyRequest { privacy_mode }),
                true,
            )
            .await?;
        Ok(())
    }

    /// Lists model access records for the authenticated organization.
    pub async fn org_models(&self) -> Result<Vec<OrgModelAccess>> {
        #[derive(Deserialize)]
        struct OrgModelsData {
            models: Vec<OrgModelAccess>,
        }
        let envelope: ApiEnvelope<OrgModelsData> = self
            .request_json(Method::GET, "/orgs/me/models", Option::<&Value>::None, true)
            .await?;
        Ok(envelope.data.models)
    }

    /// Enables or disables one model for the authenticated organization.
    pub async fn set_org_model_enabled(&self, model_id: &str, enabled: bool) -> Result<()> {
        #[derive(Serialize)]
        struct ModelEnabledRequest {
            enabled: bool,
        }
        let path = format!("/orgs/me/models/{}", encode_path_segment(model_id));
        let _: Value = self
            .request_json(
                Method::PATCH,
                &path,
                Some(&ModelEnabledRequest { enabled }),
                true,
            )
            .await?;
        Ok(())
    }

    /// Lists organization-registered server-side tools.
    pub async fn list_registered_tools(&self) -> Result<Vec<RegisteredTool>> {
        #[derive(Deserialize)]
        struct ToolsData {
            tools: Vec<RegisteredTool>,
        }
        let envelope: ApiEnvelope<ToolsData> = self
            .request_json(Method::GET, "/tools", Option::<&Value>::None, true)
            .await?;
        Ok(envelope.data.tools)
    }

    /// Returns the OpenAI-compatible tool catalog generated by the API.
    pub async fn registered_tool_catalog(&self) -> Result<Vec<Tool>> {
        #[derive(Deserialize)]
        struct ToolCatalogData {
            tools: Vec<Tool>,
        }
        let envelope: ApiEnvelope<ToolCatalogData> = self
            .request_json(Method::GET, "/tools/catalog", Option::<&Value>::None, true)
            .await?;
        Ok(envelope.data.tools)
    }

    /// Creates a server-side organization tool.
    pub async fn create_registered_tool(
        &self,
        definition: &ToolDefinition,
    ) -> Result<RegisteredTool> {
        #[derive(Deserialize)]
        struct ToolData {
            tool: RegisteredTool,
        }
        let envelope: ApiEnvelope<ToolData> = self
            .request_json(Method::POST, "/tools", Some(definition), true)
            .await?;
        Ok(envelope.data.tool)
    }

    /// Retrieves one server-side organization tool by ID.
    pub async fn get_registered_tool(&self, tool_id: &str) -> Result<RegisteredTool> {
        #[derive(Deserialize)]
        struct ToolData {
            tool: RegisteredTool,
        }
        let path = format!("/tools/{}", encode_path_segment(tool_id));
        let envelope: ApiEnvelope<ToolData> = self
            .request_json(Method::GET, &path, Option::<&Value>::None, true)
            .await?;
        Ok(envelope.data.tool)
    }

    /// Applies a partial update to one server-side organization tool.
    pub async fn update_registered_tool(
        &self,
        tool_id: &str,
        update: &ToolUpdate,
    ) -> Result<RegisteredTool> {
        #[derive(Deserialize)]
        struct ToolData {
            tool: RegisteredTool,
        }
        let path = format!("/tools/{}", encode_path_segment(tool_id));
        let envelope: ApiEnvelope<ToolData> = self
            .request_json(Method::PATCH, &path, Some(update), true)
            .await?;
        Ok(envelope.data.tool)
    }

    /// Deactivates one server-side organization tool.
    pub async fn delete_registered_tool(&self, tool_id: &str) -> Result<()> {
        let path = format!("/tools/{}", encode_path_segment(tool_id));
        let _: Value = self
            .request_json(Method::DELETE, &path, Option::<&Value>::None, true)
            .await?;
        Ok(())
    }

    /// Returns the session billing plans payload as typed JSON because plan
    /// presentation fields are intentionally extensible server metadata.
    pub async fn billing_plans(&self) -> Result<Value> {
        let envelope: ApiEnvelope<Value> = self
            .request_json(Method::GET, "/billing/plans", Option::<&Value>::None, true)
            .await?;
        Ok(envelope.data)
    }

    /// Returns the current billing summary.
    pub async fn billing_summary(&self) -> Result<Value> {
        let response: Value = self
            .request_json(
                Method::GET,
                "/billing/summary",
                Option::<&Value>::None,
                true,
            )
            .await?;
        Ok(response)
    }

    /// Returns the billing overview and recent usage snapshot.
    pub async fn billing_overview(&self) -> Result<Value> {
        let response: Value = self
            .request_json(
                Method::GET,
                "/billing/overview",
                Option::<&Value>::None,
                true,
            )
            .await?;
        Ok(response)
    }

    /// Creates a hosted checkout session for a credit package.
    pub async fn create_checkout_session(
        &self,
        request: &CheckoutRequest<'_>,
    ) -> Result<BillingActionResponse> {
        self.request_json(
            Method::POST,
            "/billing/checkout-session",
            Some(request),
            true,
        )
        .await
    }

    /// Creates a hosted checkout session for a paid subscription plan.
    pub async fn create_plan_checkout(
        &self,
        request: &PlanCheckoutRequest<'_>,
    ) -> Result<BillingActionResponse> {
        self.request_json(Method::POST, "/billing/plan-checkout", Some(request), true)
            .await
    }

    /// Opens the provider portal flow used to change the current subscription.
    pub async fn change_plan(
        &self,
        request: &PlanChangeRequest<'_>,
    ) -> Result<BillingActionResponse> {
        self.request_json(Method::POST, "/billing/plan-change", Some(request), true)
            .await
    }

    /// Creates a hosted billing portal session.
    pub async fn create_portal_session(
        &self,
        request: &PortalSessionRequest<'_>,
    ) -> Result<BillingActionResponse> {
        self.request_json(Method::POST, "/billing/portal-session", Some(request), true)
            .await
    }

    /// Retrieves the status of a hosted checkout session. Passing `None`
    /// asks the API for the most recent matching session.
    pub async fn checkout_session_status(
        &self,
        session_id: Option<&str>,
    ) -> Result<BillingActionResponse> {
        let path = session_id.map_or_else(
            || "/billing/checkout-session/status".to_string(),
            |session_id| {
                format!(
                    "/billing/checkout-session/status?sessionId={}",
                    encode_path_segment(session_id)
                )
            },
        );
        self.request_json(Method::GET, &path, Option::<&Value>::None, true)
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

    /// Creates a platform (`rk_live_...`) API key for the authenticated session.
    pub async fn create_platform_api_key(&self, name: &str) -> Result<CreatedApiKey> {
        #[derive(Serialize)]
        struct CreatePlatformKeyRequest<'a> {
            name: &'a str,
        }
        self.request_json(
            Method::POST,
            "/keys/platform",
            Some(&CreatePlatformKeyRequest { name }),
            true,
        )
        .await
    }

    /// Validates a platform/user API-key pair without sending either key in a URL.
    pub async fn validate_api_keys(
        &self,
        platform_key: &str,
        user_key: &str,
    ) -> Result<ApiKeyValidationResponse> {
        #[derive(Serialize)]
        struct ValidateKeysRequest<'a> {
            #[serde(rename = "platformKey")]
            platform_key: &'a str,
            #[serde(rename = "userKey")]
            user_key: &'a str,
        }
        let envelope: ApiEnvelope<ApiKeyValidationResponse> = self
            .request_json(
                Method::POST,
                "/keys/validate",
                Some(&ValidateKeysRequest {
                    platform_key,
                    user_key,
                }),
                false,
            )
            .await?;
        Ok(envelope.data)
    }

    /// Deletes an API key by ID.
    ///
    /// Returns the server JSON response as-is to avoid over-expanding the SDK surface.
    pub async fn delete_api_key(&self, id: &str) -> Result<serde_json::Value> {
        self.request_json(
            Method::DELETE,
            &format!("/keys/{}", encode_path_segment(id)),
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
        assert_eq!(parsed.refresh_token.as_deref(), Some("r"));
        assert_eq!(parsed.user.email, "e@x.com");
    }
}
