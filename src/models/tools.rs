//! Session-managed registered tool types.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

/// Authentication mode for a registered server-side tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ToolAuthType {
    /// No credential is injected.
    #[default]
    None,
    /// A bearer credential is injected by the service.
    Bearer,
    /// An API-key credential is injected by the service.
    ApiKey,
}

/// Tool definition accepted by the session `/tools` route.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolDefinition {
    /// Stable tool name.
    pub name: String,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Remote tool endpoint.
    pub endpoint: String,
    /// Credential injection mode.
    #[serde(rename = "authType", default)]
    pub auth_type: ToolAuthType,
    /// Tool metadata.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, Value>,
}

impl ToolDefinition {
    /// Creates a registered tool definition.
    pub fn new(name: impl Into<String>, endpoint: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            endpoint: endpoint.into(),
            auth_type: ToolAuthType::None,
            metadata: BTreeMap::new(),
        }
    }

    /// Sets a description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets credential injection mode.
    pub fn with_auth_type(mut self, auth_type: ToolAuthType) -> Self {
        self.auth_type = auth_type;
        self
    }
}

/// A registered tool returned by the session API.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RegisteredTool {
    /// Tool identifier.
    pub id: String,
    /// Tool definition fields.
    #[serde(flatten)]
    pub definition: ToolDefinition,
    /// Additional server fields.
    #[serde(flatten, default)]
    pub extra: BTreeMap<String, Value>,
}

/// Partial update body for a registered tool.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ToolUpdate {
    /// Replacement name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Replacement description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Replacement endpoint.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    /// Replacement credential mode.
    #[serde(rename = "authType", skip_serializing_if = "Option::is_none")]
    pub auth_type: Option<ToolAuthType>,
    /// Replacement metadata.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<BTreeMap<String, Value>>,
}
