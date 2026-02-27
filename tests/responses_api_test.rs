use rainy_sdk::{
    model_constants::OPENAI_GPT_5, CapabilityFlag, RainyCapabilities, RainyClient, ResponsesRequest,
};

#[test]
fn test_responses_request_serialization_supports_reasoning_and_responses_tools() {
    let request = ResponsesRequest::text(OPENAI_GPT_5, "hello")
        .with_reasoning_effort("medium")
        .with_max_output_tokens(512)
        .add_function_tool(
            "web_search",
            "Search web",
            serde_json::json!({
                "type": "object",
                "properties": { "q": { "type": "string" } },
                "required": ["q"]
            }),
        );

    let json = serde_json::to_value(&request).expect("serialize request");

    assert_eq!(json["model"], "gpt-5");
    assert_eq!(json["input"], "hello");
    assert_eq!(json["reasoning"]["effort"], "medium");
    assert_eq!(json["max_output_tokens"], 512);
    assert_eq!(json["tools"][0]["type"], "function");
    assert_eq!(json["tools"][0]["name"], "web_search");
}

#[test]
fn test_models_catalog_capabilities_deserialization() {
    let payload = serde_json::json!({
        "reasoning": "unknown",
        "image_input": true,
        "tools": true,
        "response_format": true
    });

    let capabilities: RainyCapabilities =
        serde_json::from_value(payload).expect("deserialize capabilities");

    match capabilities.reasoning {
        Some(CapabilityFlag::Text(value)) => assert_eq!(value, "unknown"),
        other => panic!("unexpected reasoning capability: {other:?}"),
    }

    match capabilities.image_input {
        Some(CapabilityFlag::Bool(value)) => assert!(value),
        other => panic!("unexpected image_input capability: {other:?}"),
    }
}

#[test]
fn test_responses_api_surface_exists() {
    let client = RainyClient::with_api_key(format!("ra-{}", "c".repeat(48)))
        .expect("failed to build client");

    let request = ResponsesRequest::text(OPENAI_GPT_5, "ping");
    let _create_response_future = client.create_response(request.clone());
    let _create_response_envelope_future = client.create_response_envelope(request.clone());
    let _create_response_stream_future = client.create_response_stream(request);
    let _models_catalog_future = client.get_models_catalog();
}
