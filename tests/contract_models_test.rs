use rainy_sdk::{
    AnthropicMessageStreamEvent, AnthropicMessageStreamEventType, ApiRouteClass, ApiSupport,
    AuthConfig, ChatCompletionRequest, ChatMessage, CreatedApiKey, ModelCatalogItem,
    RainyCapabilitiesV2, RainyClient, RainyMultimodalCapabilitiesV2, RainyParametersCapabilitiesV2,
    RainyReasoningCapabilitiesV2, ReasoningConfig, ReasoningControl, ReasoningControls,
    ReasoningEffort, ReasoningProfile, ReasoningProvider, ResponseFormat, ResponsesEvent,
    ResponsesEventType, SessionConfig, ThinkingLevel, ToolChoice, ToolFunction, ToolType,
};
use serde::Deserialize;
use serde_json::json;

fn standard_key() -> String {
    format!("ra-{}", "a".repeat(48))
}

fn platform_key() -> String {
    format!("rk_live_{}", "a".repeat(48))
}

#[test]
fn validates_both_current_api_key_formats_without_cross_prefix_acceptance() {
    assert!(AuthConfig::new(standard_key()).validate().is_ok());
    assert!(AuthConfig::new(platform_key()).validate().is_ok());

    assert!(
        AuthConfig::new(format!("ra-{}", "a".repeat(47)))
            .validate()
            .is_err()
    );
    assert!(
        AuthConfig::new(format!("rk_live_{}", "a".repeat(47)))
            .validate()
            .is_err()
    );
    assert!(
        AuthConfig::new(format!("ra-{}", "a".repeat(48) + "\n"))
            .validate()
            .is_err()
    );
}

#[test]
fn rejects_unsafe_authenticated_service_urls_but_allows_loopback_test_servers() {
    assert!(
        AuthConfig::new(standard_key())
            .with_base_url("http://127.0.0.1:3000")
            .validate()
            .is_ok()
    );
    assert!(
        AuthConfig::new(standard_key())
            .with_base_url("http://api.example.test")
            .validate()
            .is_err()
    );
    assert!(
        AuthConfig::new(standard_key())
            .with_base_url("https://api.example.test/?api_key=secret")
            .validate()
            .is_err()
    );
    assert!(
        AuthConfig::new(standard_key())
            .with_base_url("https://user:password@api.example.test")
            .validate()
            .is_err()
    );
    assert!(
        RainyClient::with_config(
            AuthConfig::new(standard_key()).with_base_url("https://api.example.test/#fragment")
        )
        .is_err()
    );
    assert!(
        rainy_sdk::RainySessionClient::with_config(
            SessionConfig::new().with_base_url("https://user:password@api.example.test")
        )
        .is_err()
    );
    assert!(
        AuthConfig::new(standard_key())
            .with_base_url(" https://api.example.test")
            .validate()
            .is_err()
    );
    assert!(
        AuthConfig::new(standard_key())
            .with_timeout(0)
            .validate()
            .is_err()
    );
}

#[test]
fn serializes_reasoning_forms_exactly_and_keeps_legacy_setters_on_wire() {
    let enabled = serde_json::to_value(
        ChatCompletionRequest::new("provider/model", vec![ChatMessage::user("hi")])
            .with_reasoning(true),
    )
    .expect("enabled JSON");
    assert_eq!(enabled["reasoning"], json!(true));

    let disabled = serde_json::to_value(
        ChatCompletionRequest::new("provider/model", vec![ChatMessage::user("hi")])
            .with_reasoning(false),
    )
    .expect("disabled JSON");
    assert_eq!(disabled["reasoning"], json!(false));

    let effort = serde_json::to_value(
        ChatCompletionRequest::new("provider/model", vec![ChatMessage::user("hi")])
            .with_reasoning_effort(ReasoningEffort::High),
    )
    .expect("effort JSON");
    assert_eq!(effort["reasoning"], json!({"effort": "high"}));

    let budget = serde_json::to_value(
        ChatCompletionRequest::new("provider/model", vec![ChatMessage::user("hi")])
            .with_reasoning_budget(1024),
    )
    .expect("budget JSON");
    assert_eq!(budget["reasoning"], json!({"max_tokens": 1024}));

    let legacy_level = serde_json::to_value(
        ChatCompletionRequest::new("gemini-3-flash", vec![ChatMessage::user("hi")])
            .with_thinking_level(ThinkingLevel::High)
            .with_include_thoughts(true),
    )
    .expect("legacy level JSON");
    assert_eq!(legacy_level["reasoning"], json!({"effort": "high"}));
    assert_eq!(legacy_level["include_reasoning"], json!(true));

    let legacy_dynamic = serde_json::to_value(
        ChatCompletionRequest::new("gemini-2.5-flash", vec![ChatMessage::user("hi")])
            .with_thinking_budget(-1),
    )
    .expect("legacy dynamic JSON");
    assert_eq!(legacy_dynamic["reasoning"], json!({"max_tokens": -1}));

    let empty = serde_json::to_value(
        ChatCompletionRequest::new("provider/model", vec![ChatMessage::user("hi")])
            .with_reasoning_config(ReasoningConfig::adaptive()),
    )
    .expect("adaptive JSON");
    assert_eq!(empty["reasoning"], json!({"enabled": true}));
}

#[test]
fn serializes_structured_tool_and_multimodal_controls_without_shape_changes() {
    let format = serde_json::to_value(ResponseFormat::JsonSchema {
        json_schema: json!({
            "name": "answer",
            "strict": true,
            "schema": {"type": "object"}
        }),
    })
    .expect("response format JSON");
    assert_eq!(
        format,
        json!({
            "type": "json_schema",
            "json_schema": {
                "name": "answer",
                "strict": true,
                "schema": {"type": "object"}
            }
        })
    );

    let choice = serde_json::to_value(ToolChoice::Tool {
        r#type: ToolType::Function,
        function: ToolFunction {
            name: "lookup".to_string(),
        },
    })
    .expect("tool choice JSON");
    assert_eq!(
        choice,
        json!({"type": "function", "function": {"name": "lookup"}})
    );

    let request = ChatCompletionRequest::new("provider/model", vec![ChatMessage::user("hi")])
        .with_stop("END")
        .with_provider("openai")
        .with_modalities(["text", "audio"])
        .with_chat_stream_options(rainy_sdk::ChatStreamOptions::include_usage());
    let value = serde_json::to_value(request).expect("chat JSON");
    assert_eq!(value["stop"], json!("END"));
    assert_eq!(value["provider"], json!({"order": ["openai"]}));
    assert_eq!(value["modalities"], json!(["text", "audio"]));
    assert_eq!(value["stream_options"], json!({"include_usage": true}));
}

#[test]
fn capability_metadata_builds_the_declared_nested_reasoning_path() {
    let model = ModelCatalogItem {
        id: "google/gemini-next".to_string(),
        rainy_capabilities_v2: Some(RainyCapabilitiesV2 {
            multimodal: RainyMultimodalCapabilitiesV2::default(),
            reasoning: RainyReasoningCapabilitiesV2 {
                supported: true,
                controls: Some(ReasoningControls {
                    reasoning_effort: Some(true),
                    effort: Some(vec!["low".to_string(), "high".to_string()]),
                    thinking_budget: Some(rainy_sdk::ThinkingBudget {
                        min: -1,
                        max: 32768,
                        dynamic_value: Some(-1),
                        disable_value: Some(0),
                    }),
                    ..Default::default()
                }),
                profiles: vec![
                    ReasoningProfile {
                        provider: ReasoningProvider::Google,
                        parameter_path: "reasoning.effort".to_string(),
                        values: Some(vec!["low".to_string(), "high".to_string()]),
                        notes: None,
                    },
                    ReasoningProfile {
                        provider: ReasoningProvider::Google,
                        parameter_path: "reasoning.max_tokens".to_string(),
                        values: None,
                        notes: None,
                    },
                ],
                ..Default::default()
            },
            parameters: RainyParametersCapabilitiesV2::default(),
        }),
        ..Default::default()
    };

    let effort = model
        .reasoning_config_for(&ReasoningControl::Effort(ReasoningEffort::High))
        .expect("declared effort");
    assert_eq!(
        serde_json::to_value(effort).unwrap(),
        json!({"effort": "high"})
    );

    let dynamic = model
        .reasoning_budget_config(rainy_sdk::ReasoningBudget::Dynamic)
        .expect("declared dynamic budget");
    assert_eq!(
        serde_json::to_value(dynamic).unwrap(),
        json!({"max_tokens": -1})
    );

    assert!(
        model
            .reasoning_config_for(&ReasoningControl::Effort(ReasoningEffort::Max))
            .is_err()
    );
}

#[test]
fn classifies_native_stream_events_and_preserves_unknown_provider_values() {
    assert_eq!(
        ResponsesEventType::from_name(Some("RESPONSE.OUTPUT_TEXT.DELTA")),
        ResponsesEventType::OutputTextDelta
    );
    let response_event = ResponsesEvent::new(
        Some("response.function_call_arguments.delta".to_string()),
        json!({"delta": "{}"}),
    );
    assert_eq!(
        response_event.kind,
        ResponsesEventType::FunctionCallArgumentsDelta
    );
    assert_eq!(response_event.text_delta(), Some("{}"));

    let deserialized_response_event: ResponsesEvent = serde_json::from_value(json!({
        "event": "response.output_text.delta",
        "data": {"delta": "classified"}
    }))
    .expect("deserialize Responses event");
    assert_eq!(
        deserialized_response_event.kind,
        ResponsesEventType::OutputTextDelta
    );

    let message_event = AnthropicMessageStreamEvent::new(
        Some("content_block_delta".to_string()),
        json!({"type": "content_block_delta"}),
    );
    assert_eq!(
        message_event.kind,
        AnthropicMessageStreamEventType::ContentBlockDelta
    );

    let deserialized_message_event: AnthropicMessageStreamEvent = serde_json::from_value(json!({
        "event": "message_stop",
        "data": {"type": "message_stop"}
    }))
    .expect("deserialize Messages event");
    assert_eq!(
        deserialized_message_event.kind,
        AnthropicMessageStreamEventType::MessageStop
    );

    let provider: ReasoningProvider = serde_json::from_value(json!("future_provider"))
        .expect("unknown provider remains representable");
    assert_eq!(
        provider,
        ReasoningProvider::Custom("future_provider".to_string())
    );

    let chunk_shaped_payload = json!({
        "id": "chatcmpl_future",
        "object": "chat.completion.chunk",
        "created": 1,
        "model": "provider/model",
        "choices": []
    });
    assert!(matches!(
        rainy_sdk::ChatStreamEvent::from_sse_event(
            Some("future.provider.event"),
            chunk_shaped_payload
        ),
        rainy_sdk::ChatStreamEvent::Unknown { event, .. }
            if event == "future.provider.event"
    ));
}

#[test]
fn capability_matrix_is_unique_and_explicit_about_unsupported_routes() {
    let matrix = rainy_sdk::capability_matrix();
    for (index, left) in matrix.iter().enumerate() {
        assert!(
            matrix[index + 1..]
                .iter()
                .all(|right| (left.method, left.path) != (right.method, right.path)),
            "duplicate capability route: {} {}",
            left.method,
            left.path
        );
    }
    let chat = matrix
        .iter()
        .find(|route| route.path == "/api/v1/chat/completions")
        .expect("chat route");
    assert_eq!(chat.class, ApiRouteClass::Public);
    assert_eq!(chat.support, ApiSupport::Implemented);
    assert!(chat.streaming);

    let refresh = matrix
        .iter()
        .find(|route| route.path == "/api/v1/auth/refresh")
        .expect("refresh route");
    assert_eq!(refresh.support, ApiSupport::KnownUnsupported);

    let agents = matrix
        .iter()
        .find(|route| route.path == "/api/v1/agents")
        .expect("agents route");
    assert_eq!(agents.class, ApiRouteClass::Unsupported);
    assert_eq!(agents.support, ApiSupport::KnownUnsupported);
}

#[test]
fn packaged_capability_matrix_matches_the_runtime_route_inventory() {
    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct DocumentedRoute {
        method: String,
        path: String,
        class: ApiRouteClass,
        support: ApiSupport,
        auth: String,
        streaming: bool,
    }

    let documented: Vec<DocumentedRoute> =
        serde_json::from_str(include_str!("../docs/API_CAPABILITY_MATRIX.json"))
            .expect("capability matrix JSON");
    let runtime = rainy_sdk::capability_matrix()
        .iter()
        .map(|route| DocumentedRoute {
            method: route.method.to_string(),
            path: route.path.to_string(),
            class: route.class,
            support: route.support,
            auth: route.auth.to_string(),
            streaming: route.streaming,
        })
        .collect::<Vec<_>>();

    assert_eq!(documented, runtime);
}

#[test]
fn debug_output_redacts_session_credentials_and_created_key_material() {
    let request = rainy_sdk::session::LoginRequest {
        email: "user@example.com",
        password: "correct-horse-battery-staple",
    };
    let request_debug = format!("{request:?}");
    assert!(!request_debug.contains("correct-horse"));
    assert!(request_debug.contains("REDACTED"));

    let key = CreatedApiKey {
        key: standard_key(),
        id: "key_1".to_string(),
        name: "test".to_string(),
        r#type: "standard".to_string(),
    };
    let key_debug = format!("{key:?}");
    assert!(!key_debug.contains(&standard_key()));
    assert!(key_debug.contains("REDACTED"));

    let auth_debug = format!("{:?}", AuthConfig::new(standard_key()));
    assert!(!auth_debug.contains(&standard_key()));

    let login: rainy_sdk::LoginResponse = serde_json::from_value(json!({
        "accessToken": "access-secret",
        "refreshToken": "refresh-secret",
        "user": {"id": "user_1", "email": "user@example.com", "role": "member"},
        "data": {"accessToken": "nested-access-secret"}
    }))
    .expect("login response");
    let login_debug = format!("{login:?}");
    assert!(!login_debug.contains("access-secret"));
    assert!(!login_debug.contains("nested-access-secret"));
    assert!(login_debug.contains("REDACTED"));

    let refresh: rainy_sdk::RefreshResponse = serde_json::from_value(json!({
        "accessToken": "rotated-access-secret",
        "refreshToken": "rotated-refresh-secret",
        "data": {"accessToken": "nested-rotated-access-secret"}
    }))
    .expect("refresh response");
    let refresh_debug = format!("{refresh:?}");
    assert!(!refresh_debug.contains("rotated-access-secret"));
    assert!(!refresh_debug.contains("nested-rotated-access-secret"));
    assert!(refresh_debug.contains("REDACTED"));

    let mut session = rainy_sdk::RainySessionClient::with_config(
        SessionConfig::new().with_base_url("http://localhost:3000"),
    )
    .expect("loopback session client");
    session.set_access_token("session-secret");
    let session_debug = format!("{session:?}");
    assert!(!session_debug.contains("session-secret"));
    assert!(session_debug.contains("REDACTED"));

    let _ = RainyClient::with_api_key(standard_key()).expect("valid API-key client");
}
