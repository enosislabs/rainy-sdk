use rainy_sdk::{
    AuthConfig, ChatCompletionRequest, ChatMessage, ModelCatalogItem, ModelSelectionCriteria,
    RainyError, ReasoningConfig, ReasoningEffort, ReasoningPreference, ResponsesRequest,
    RetryConfig, build_reasoning_config, select_models,
};

#[cfg(feature = "rainy-account")]
use rainy_sdk::{RainySessionClient, SessionConfig};

#[test]
fn auth_accepts_protocol_neutral_keys() {
    assert!(AuthConfig::new("sk-compatible-key").validate().is_ok());
    assert!(AuthConfig::new("").validate().is_err());
    assert!(AuthConfig::new("key with whitespace").validate().is_err());
}

#[test]
fn auth_rejects_unsafe_service_urls() {
    let key = "sk-compatible-key";
    assert!(
        AuthConfig::new(key)
            .with_base_url("http://example.com")
            .validate()
            .is_err()
    );
    assert!(
        AuthConfig::new(key)
            .with_base_url("https://user:password@example.com")
            .validate()
            .is_err()
    );
    assert!(
        AuthConfig::new(key)
            .with_base_url("https://example.com/v1?token=secret")
            .validate()
            .is_err()
    );
    assert!(
        AuthConfig::new(key)
            .with_base_url("http://127.0.0.1:3000")
            .validate()
            .is_ok()
    );
}

#[test]
fn secret_does_not_appear_in_debug_or_display() {
    let secret = "sk-this-must-not-be-printed";
    let config = AuthConfig::new(secret);
    assert!(!format!("{config:?}").contains(secret));
    assert!(!config.to_string().contains(secret));
}

#[test]
fn compact_chat_builder_and_validation_are_protocol_only() {
    let messages = vec![ChatMessage::user("hello")];
    let request = ChatCompletionRequest::new("any/model", messages.clone())
        .with_temperature(0.7)
        .with_max_tokens(100)
        .with_user("user-1")
        .with_reasoning_effort(ReasoningEffort::High)
        .with_include_reasoning(true);

    assert_eq!(request.messages, messages);
    assert_eq!(request.temperature, Some(0.7));
    assert_eq!(request.reasoning_effort, Some(ReasoningEffort::High));
    assert!(request.validate_openai_compatibility().is_ok());

    let zero_choices = ChatCompletionRequest::new("any/model", vec![]).with_n(0);
    assert!(zero_choices.validate_openai_compatibility().is_err());
}

#[test]
fn reasoning_effort_and_explicit_budget_are_not_converted() {
    let effort = serde_json::to_value(ReasoningConfig::effort(ReasoningEffort::XHigh)).unwrap();
    assert_eq!(effort, serde_json::json!({"effort": "xhigh"}));

    let budget = serde_json::to_value(ReasoningConfig::manual_budget(2048)).unwrap();
    assert_eq!(budget, serde_json::json!({"max_tokens": 2048}));

    let request = ResponsesRequest::text("any/model", "hello")
        .with_reasoning_effort(ReasoningEffort::High)
        .with_reasoning_budget(4096);
    let value = serde_json::to_value(request).unwrap();
    assert_eq!(value["reasoning_effort"], "high");
    assert_eq!(value["reasoning"]["max_tokens"], 4096);
}

#[test]
fn catalog_selection_uses_public_capabilities_only() {
    let model = ModelCatalogItem {
        id: "any/reasoning-model".to_string(),
        supported_parameters: Some(vec!["reasoning_effort".to_string()]),
        ..Default::default()
    };
    let selected = select_models(
        std::slice::from_ref(&model),
        &ModelSelectionCriteria {
            require_reasoning: Some(true),
            ..Default::default()
        },
    );
    assert_eq!(selected.len(), 1);
    let config = build_reasoning_config(&model, &ReasoningPreference::effort("high"));
    assert_eq!(config, Some(serde_json::json!({"effort": "high"})));
}

#[test]
fn retry_config_is_monotonic_without_jitter() {
    let mut config = RetryConfig::new(5);
    config.jitter = false;
    let delay0 = config.delay_for_attempt(0);
    let delay1 = config.delay_for_attempt(1);
    let delay2 = config.delay_for_attempt(2);
    assert!(delay1 >= delay0);
    assert!(delay2 >= delay1);
    assert!(delay2.as_millis() <= config.max_delay_ms as u128);
}

#[test]
fn error_helpers_remain_safe_and_typed() {
    let auth_error = RainyError::Authentication {
        code: "INVALID_KEY".to_string(),
        message: "Invalid key".to_string(),
        retryable: false,
    };
    assert!(!auth_error.is_access_denied());
    assert!(!auth_error.is_retryable());
    assert_eq!(auth_error.code(), Some("INVALID_KEY"));
    assert_eq!(auth_error.request_id(), None);

    let access_denied = RainyError::AccessDenied {
        code: "CAPABILITY_DENIED".to_string(),
        message: "The requested capability is unavailable".to_string(),
        details: None,
    };
    assert!(access_denied.is_access_denied());
    assert!(!access_denied.is_retryable());
    assert_eq!(access_denied.code(), Some("CAPABILITY_DENIED"));

    let rate_limit = RainyError::RateLimit {
        code: "RATE_LIMIT_EXCEEDED".to_string(),
        message: "Too many requests".to_string(),
        retry_after: Some(4),
        current_usage: None,
    };
    assert!(rate_limit.is_retryable());
    assert_eq!(rate_limit.retry_after(), Some(4));
}

#[cfg(feature = "rainy-account")]
#[test]
fn account_client_is_an_explicit_opt_in() {
    let client = RainySessionClient::with_config(
        SessionConfig::new()
            .with_base_url("http://localhost:3000")
            .with_timeout(15),
    )
    .expect("session client");
    assert_eq!(client.base_url(), "http://localhost:3000");
    assert!(client.access_token().is_none());
}
