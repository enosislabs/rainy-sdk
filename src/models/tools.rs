//! Stable tool and structured-output models.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// OpenAI-compatible response format.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ResponseFormat {
    /// Unconstrained text output.
    Text,
    /// A JSON object without a named schema.
    JsonObject,
    /// A named JSON schema supplied by the caller.
    JsonSchema {
        /// Schema object accepted by the compatible endpoint.
        json_schema: Value,
    },
}

/// OpenAI-compatible tool wrapper.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Tool {
    /// Tool kind, normally `function`.
    pub r#type: ToolType,
    /// Function definition.
    pub function: FunctionDefinition,
}

/// Known tool kinds.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolType {
    /// A function tool.
    Function,
}

/// OpenAI-compatible function definition.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FunctionDefinition {
    /// Function name.
    pub name: String,
    /// Human-readable description.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// JSON Schema for function parameters.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Value>,
}

impl FunctionDefinition {
    /// Creates a function definition.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: None,
            parameters: None,
        }
    }

    /// Sets the description.
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Sets the open JSON Schema payload.
    pub fn with_parameters(mut self, parameters: Value) -> Self {
        self.parameters = Some(parameters);
        self
    }
}

/// OpenAI-compatible tool choice.
#[derive(Debug, Clone, PartialEq)]
pub enum ToolChoice {
    /// Let the model choose whether to call a tool.
    Auto,
    /// Do not call a tool.
    None,
    /// Require a tool call.
    Required,
    /// Force one named function.
    Tool {
        /// Tool type.
        r#type: ToolType,
        /// Function selector.
        function: ToolFunction,
    },
    /// Preserve an endpoint-specific choice object.
    Raw(Value),
}

/// Function selector inside a tool-choice object.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolFunction {
    /// Function name.
    pub name: String,
}

impl ToolChoice {
    /// Forces the model to call a named function.
    pub fn function(name: impl Into<String>) -> Self {
        Self::Tool {
            r#type: ToolType::Function,
            function: ToolFunction { name: name.into() },
        }
    }
}

// OpenAI uses string values for the three simple choices. Keep the public enum
// pleasant to use while emitting the exact wire shape.
impl serde::Serialize for ToolChoice {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Self::Auto => serializer.serialize_str("auto"),
            Self::None => serializer.serialize_str("none"),
            Self::Required => serializer.serialize_str("required"),
            Self::Tool { r#type, function } => {
                #[derive(Serialize)]
                struct Choice<'a> {
                    r#type: &'a ToolType,
                    function: &'a ToolFunction,
                }
                Choice { r#type, function }.serialize(serializer)
            }
            Self::Raw(value) => value.serialize(serializer),
        }
    }
}

impl<'de> serde::Deserialize<'de> for ToolChoice {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::String(value) => match value.as_str() {
                "auto" => Ok(Self::Auto),
                "none" => Ok(Self::None),
                "required" => Ok(Self::Required),
                _ => Ok(Self::Raw(Value::String(value))),
            },
            Value::Object(value) => {
                #[derive(Deserialize)]
                struct Choice {
                    r#type: ToolType,
                    function: ToolFunction,
                }
                let choice = serde_json::from_value::<Choice>(Value::Object(value))
                    .map_err(|error| serde::de::Error::custom(error.to_string()))?;
                Ok(Self::Tool {
                    r#type: choice.r#type,
                    function: choice.function,
                })
            }
            other => Ok(Self::Raw(other)),
        }
    }
}
