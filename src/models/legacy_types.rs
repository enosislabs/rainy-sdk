use super::{HealthStatus, MessageRole, Usage};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents a user account (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub user_id: String,
    pub plan_name: String,
    pub current_credits: f64,
    pub credits_used_this_month: f64,
    pub credits_reset_date: DateTime<Utc>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

/// Represents an API key (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: Uuid,
    pub key: String,
    pub owner_id: Uuid,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Represents usage statistics over a period (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsageStats {
    pub period_days: u32,
    pub daily_usage: Vec<DailyUsage>,
    pub recent_transactions: Vec<CreditTransaction>,
    pub total_requests: u64,
    pub total_tokens: u64,
}

/// Represents usage data for a single day (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyUsage {
    pub date: String,
    pub credits_used: f64,
    pub requests: u64,
    pub tokens: u64,
}

/// Represents a single credit transaction (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreditTransaction {
    pub id: Uuid,
    pub transaction_type: TransactionType,
    pub credits_amount: f64,
    pub credits_balance_after: f64,
    pub provider: Option<String>,
    pub model: Option<String>,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

/// The type of credit transaction (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Usage,
    Reset,
    Purchase,
    Refund,
}

/// A legacy type alias for `MessageRole`.
pub type ChatRole = MessageRole;
/// A legacy type alias for `Usage`.
pub type ChatUsage = Usage;
/// A legacy type alias for `HealthStatus`.
pub type HealthCheck = HealthStatus;

/// Represents the status of backend services (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthServices {
    pub database: bool,
    pub redis: bool,
    pub providers: bool,
}

/// The health status of the API (legacy).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatusEnum {
    Healthy,
    Degraded,
    Unhealthy,
    NeedsInit,
}
