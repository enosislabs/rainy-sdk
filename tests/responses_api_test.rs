use rainy_sdk::{
    CapabilityFlag, EmbeddingEncodingFormat, EmbeddingValue, EmbeddingsRequest, EmbeddingsResponse,
    ModelArchitecture, ModelCatalogItem, ModelPricing, ModelSelectionCriteria, RainyCapabilities,
    RainyClient, ReasoningEffort, ReasoningPreference, ResponsesApiResponse, ResponsesEvent,
    ResponsesEventType, ResponsesRequest, build_reasoning_config, select_models,
};

#[test]
fn responses_request_serializes_reasoning_and_open_tools() {
    let request = ResponsesRequest::text("compatible/responses-model", "hello")
        .with_reasoning_effort(ReasoningEffort::Medium)
        .with_max_output_tokens(512)
        .with_max_tool_calls(4)
        .with_parallel_tool_calls(true)
        .with_tool_choice("auto")
        .add_function_tool(
            "lookup",
            "Look up a value",
            serde_json::json!({
                "type": "object",
                "properties": { "key": { "type": "string" } },
                "required": ["key"]
            }),
        )
        .add_web_search_tool();

    let json = serde_json::to_value(request).expect("serialize request");

    assert_eq!(json["model"], "compatible/responses-model");
    assert_eq!(json["input"], "hello");
    assert_eq!(json["reasoning_effort"], "medium");
    assert_eq!(json["max_output_tokens"], 512);
    assert_eq!(json["max_tool_calls"], 4);
    assert_eq!(json["parallel_tool_calls"], true);
    assert_eq!(json["tools"][0]["type"], "function");
    assert_eq!(json["tools"][1]["type"], "web_search");
}

#[test]
fn responses_request_keeps_explicit_numeric_budget_separate() {
    let request = ResponsesRequest::text("compatible/reasoning-model", "think")
        .with_reasoning_budget(4096)
        .with_include_reasoning(true);
    let json = serde_json::to_value(request).expect("serialize budget request");

    assert_eq!(json["reasoning"]["max_tokens"], 4096);
    assert_eq!(json["include_reasoning"], true);
    assert!(json.get("reasoning_effort").is_none());
}

#[test]
fn responses_request_supports_continuation_and_multimodal_input() {
    let input = serde_json::json!([
        {"type": "input_text", "text": "Describe this image."},
        {"type": "input_image", "image_url": "https://example.com/image.png", "detail": "high"},
        ResponsesRequest::function_call_output("call_123", r#"{"value": 21}"#)
    ]);
    let request = ResponsesRequest::new("compatible/model", input)
        .with_previous_response_id("resp_123")
        .add_strict_function_tool(
            "lookup",
            "Look up a value",
            serde_json::json!({"type": "object", "properties": {}}),
        );
    let json = serde_json::to_value(request).expect("serialize continuation");

    assert_eq!(json["input"][0]["type"], "input_text");
    assert_eq!(json["input"][1]["type"], "input_image");
    assert_eq!(json["input"][2]["type"], "function_call_output");
    assert_eq!(json["previous_response_id"], "resp_123");
    assert_eq!(json["tools"][0]["strict"], true);
}

#[test]
fn responses_response_extracts_function_calls_and_nested_text() {
    let response: ResponsesApiResponse = serde_json::from_value(serde_json::json!({
        "id": "resp_123",
        "output": [
            {"type": "function_call", "call_id": "call_123", "name": "lookup"},
            {"type": "message", "content": [
                {"type": "output_text", "text": "hello "},
                {"type": "output_text", "text": "world"}
            ]}
        ]
    }))
    .expect("deserialize response output");

    assert_eq!(response.function_calls().len(), 1);
    assert_eq!(response.text().as_deref(), Some("hello world"));
}

#[test]
fn responses_events_preserve_native_names_and_classify_known_events() {
    let event = ResponsesEvent::new(
        Some("response.output_text.delta".to_string()),
        serde_json::json!({"delta": "hello"}),
    );
    assert_eq!(event.kind, ResponsesEventType::OutputTextDelta);
    assert_eq!(event.text_delta(), Some("hello"));

    let unknown = ResponsesEvent::new(
        Some("future.response.event".to_string()),
        serde_json::json!({"type": "response.output_text.delta", "delta": "opaque"}),
    );
    assert_eq!(unknown.kind, ResponsesEventType::Unknown);
    assert_eq!(unknown.event.as_deref(), Some("future.response.event"));
}

#[test]
fn embeddings_contract_serializes_and_deserializes() {
    let request = EmbeddingsRequest::text("compatible/embedding-model", "hello")
        .with_encoding_format(EmbeddingEncodingFormat::Float)
        .with_dimensions(256)
        .with_user("test-user");
    let value = serde_json::to_value(request).expect("serialize embeddings request");
    assert_eq!(value["input"], "hello");
    assert_eq!(value["encoding_format"], "float");
    assert_eq!(value["dimensions"], 256);

    let response: EmbeddingsResponse = serde_json::from_value(serde_json::json!({
        "object": "list",
        "data": [{"object": "embedding", "index": 0, "embedding": [0.25, -0.5]}],
        "model": "compatible/embedding-model",
        "usage": {"prompt_tokens": 1, "total_tokens": 1}
    }))
    .expect("deserialize embeddings response");
    assert!(matches!(
        response.data[0].embedding,
        EmbeddingValue::Float(_)
    ));
}

#[test]
fn public_catalog_capabilities_deserialize_and_select() {
    let capabilities: RainyCapabilities = serde_json::from_value(serde_json::json!({
        "reasoning": "preview",
        "image_input": true,
        "tools": true,
        "response_format": true
    }))
    .expect("deserialize capabilities");
    assert!(
        matches!(capabilities.reasoning, Some(CapabilityFlag::Text(value)) if value == "preview")
    );
    assert!(matches!(
        capabilities.image_input,
        Some(CapabilityFlag::Bool(true))
    ));

    let model = ModelCatalogItem {
        id: "compatible/reasoning-model".to_string(),
        context_length: Some(128_000),
        pricing: Some(ModelPricing {
            prompt: Some("0.000001".to_string()),
            completion: Some("0.000002".to_string()),
        }),
        supported_parameters: Some(vec![
            "reasoning_effort".to_string(),
            "reasoning.max_tokens".to_string(),
        ]),
        architecture: Some(ModelArchitecture {
            input_modalities: vec!["text".to_string(), "image".to_string()],
            output_modalities: vec!["text".to_string()],
            ..Default::default()
        }),
        rainy_capabilities: Some(RainyCapabilities {
            reasoning: Some(CapabilityFlag::Bool(true)),
            tools: Some(CapabilityFlag::Bool(true)),
            response_format: Some(CapabilityFlag::Bool(true)),
            ..Default::default()
        }),
        ..Default::default()
    };
    let selected = select_models(
        std::slice::from_ref(&model),
        &ModelSelectionCriteria {
            required_input_modalities: vec!["image".to_string()],
            require_tools: Some(true),
            require_reasoning: Some(true),
            ..Default::default()
        },
    );
    assert_eq!(selected.len(), 1);

    let effort = build_reasoning_config(&model, &ReasoningPreference::effort("high"));
    assert_eq!(effort, Some(serde_json::json!({"effort": "high"})));
    let budget = build_reasoning_config(&model, &ReasoningPreference::budget(1024));
    assert_eq!(budget, Some(serde_json::json!({"max_tokens": 1024})));
}

#[test]
fn public_client_surface_does_not_require_discovery() {
    let client = RainyClient::with_api_key("compatible-key").expect("client");
    let request = ResponsesRequest::text("custom/model", "ping");
    let _response = client.create_response(request.clone());
    let _envelope = client.create_response_envelope(request.clone());
    let _stream = client.create_response_stream(request);
    let _embeddings = client.create_embeddings(EmbeddingsRequest::text("custom/embed", "ping"));
}
