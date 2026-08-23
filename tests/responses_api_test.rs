use rainy_sdk::{
    CapabilityFlag, EmbeddingEncodingFormat, EmbeddingValue, EmbeddingsRequest, EmbeddingsResponse,
    ModelBillingClass, ModelCatalogItem, ModelDataPolicyStatus, ModelPricing,
    ModelSelectionCriteria, ModelTier, RainyCapabilities, RainyCapabilitiesV2, RainyClient,
    ReasoningMode, ReasoningPreference, ResponsesRequest, build_reasoning_config, select_models,
};

#[test]
fn test_responses_request_serialization_supports_reasoning_and_responses_tools() {
    let request = ResponsesRequest::text("gpt-5", "hello")
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
fn test_embeddings_contract_serialization_and_deserialization() {
    let mut request = EmbeddingsRequest::text("openai/text-embedding-3-small", "hello");
    request.encoding_format = Some(EmbeddingEncodingFormat::Float);
    request.dimensions = Some(256);
    let value = serde_json::to_value(request).expect("serialize embeddings request");
    assert_eq!(value["input"], "hello");
    assert_eq!(value["encoding_format"], "float");
    assert_eq!(value["dimensions"], 256);

    let response: EmbeddingsResponse = serde_json::from_value(serde_json::json!({
        "object": "list",
        "data": [{"object": "embedding", "index": 0, "embedding": [0.25, -0.5]}],
        "model": "openai/text-embedding-3-small",
        "usage": {"prompt_tokens": 1, "total_tokens": 1}
    }))
    .expect("deserialize embeddings response");
    assert!(matches!(
        response.data[0].embedding,
        EmbeddingValue::Float(_)
    ));
}

#[test]
fn test_responses_request_serialization_supports_gpt5_native_fields() {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("flow".to_string(), "responses".to_string());

    let request = ResponsesRequest::text("gpt-5", "hello")
        .with_instructions("Be concise")
        .with_previous_response_id("resp_123")
        .with_service_tier("auto")
        .with_include_reasoning(true)
        .with_provider_options(serde_json::json!({
            "openai": { "reasoning": { "effort": "medium" } }
        }))
        .with_metadata(metadata);

    let json = serde_json::to_value(&request).expect("serialize gpt5-native fields");
    assert_eq!(json["instructions"], "Be concise");
    assert_eq!(json["previous_response_id"], "resp_123");
    assert_eq!(json["service_tier"], "auto");
    assert_eq!(json["include_reasoning"], true);
    assert_eq!(
        json["provider_options"]["openai"]["reasoning"]["effort"],
        "medium"
    );
    assert_eq!(json["metadata"]["flow"], "responses");
}

#[test]
fn test_responses_api_response_deserializes_extended_fields() {
    let payload = serde_json::json!({
        "id": "resp_123",
        "object": "response",
        "model": "gpt-5",
        "status": "completed",
        "output_text": "done",
        "error": null,
        "incomplete_details": null,
        "usage": {
            "input_tokens": 10,
            "output_tokens": 20
        }
    });

    let response: rainy_sdk::ResponsesApiResponse =
        serde_json::from_value(payload).expect("deserialize extended response");
    assert_eq!(response.status.as_deref(), Some("completed"));
    assert_eq!(response.output_text.as_deref(), Some("done"));
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

    let request = ResponsesRequest::text("gpt-5", "ping");
    let _create_response_future = client.create_response(request.clone());
    let _create_response_envelope_future = client.create_response_envelope(request.clone());
    let _create_response_stream_future = client.create_response_stream(request);
    let _models_catalog_future = client.get_models_catalog();
    let _select_models_future = client.select_models(ModelSelectionCriteria::default());
}

#[test]
fn test_models_catalog_capabilities_v2_deserialization() {
    let payload = serde_json::json!({
        "id": "google/gemini-3.1-pro-preview",
        "context_length": 1048576,
        "pricing": { "prompt": "0.000002", "completion": "0.000012" },
        "supported_parameters": ["tools", "response_format", "reasoning"],
        "rainy_capabilities_v2": {
            "multimodal": { "input": ["text", "image"], "output": ["text"] },
            "reasoning": {
                "supported": true,
                "controls": {
                    "observed_parameters": ["reasoning", "include_reasoning", "response_format"],
                    "reasoning_toggle": true,
                    "thinking_level": ["minimal", "low", "medium", "high"],
                    "thinking_budget": { "min": -1, "max": 32768, "dynamic_value": -1, "disable_value": 0 }
                },
                "profiles": [
                    { "provider": "google", "parameter_path": "thinking_config.thinking_level" }
                ],
                "toggle": { "enable_param": "reasoning.enabled", "include_reasoning_param": "include_reasoning" }
            },
            "parameters": { "accepted": ["tools", "response_format", "reasoning"] }
        }
    });

    let item: ModelCatalogItem =
        serde_json::from_value(payload).expect("deserialize model catalog item");
    let caps = item
        .rainy_capabilities_v2
        .expect("missing rainy_capabilities_v2");
    assert!(caps.reasoning.supported);
    assert_eq!(
        caps.multimodal.input,
        vec!["text".to_string(), "image".to_string()]
    );
}

#[test]
fn test_models_catalog_deserializes_entitlement_and_policy_metadata() {
    let payload = serde_json::json!({
        "id": "provider/frontier-model",
        "rainy_model_tier": "frontier",
        "rainy_billing_class": "extreme",
        "rainy_plan_context_length": 1000000,
        "rainy_effective_context_length": 262144,
        "rainy_organization_enabled": true,
        "rainy_privacy_mode": "strict",
        "rainy_privacy_compatible": false,
        "rainy_data_policy": {
            "status": "verified",
            "requiresDataDisclosure": true,
            "notice": "Provider retains prompts",
            "trainingWithInputs": false,
            "zdrAvailable": false,
            "retentionDays": 30,
            "coveredModel": true,
            "source": "provider-policy",
            "verifiedAt": "2026-08-01T00:00:00Z",
            "expiresAt": "2026-11-01T00:00:00Z",
            "effectiveEndpoint": "provider/default"
        }
    });

    let item: ModelCatalogItem = serde_json::from_value(payload).expect("catalog metadata");
    assert_eq!(item.rainy_model_tier, Some(ModelTier::Frontier));
    assert_eq!(item.rainy_billing_class, Some(ModelBillingClass::Extreme));
    assert_eq!(item.rainy_effective_context_length, Some(262_144));
    assert_eq!(item.rainy_organization_enabled, Some(true));
    assert_eq!(item.rainy_privacy_compatible, Some(false));
    let policy = item.rainy_data_policy.expect("data policy");
    assert_eq!(policy.status, Some(ModelDataPolicyStatus::Verified));
    assert!(policy.requires_data_disclosure);
    assert_eq!(policy.retention_days, Some(30));
}

#[test]
fn test_select_models_filters_tier_and_authenticated_access() {
    let accessible = ModelCatalogItem {
        id: "model/accessible".to_string(),
        rainy_model_tier: Some(ModelTier::Premium),
        rainy_organization_enabled: Some(true),
        rainy_privacy_compatible: Some(true),
        rainy_capabilities_v2: Some(RainyCapabilitiesV2::default()),
        ..Default::default()
    };
    let privacy_blocked = ModelCatalogItem {
        id: "model/privacy-blocked".to_string(),
        rainy_model_tier: Some(ModelTier::Premium),
        rainy_organization_enabled: Some(true),
        rainy_privacy_compatible: Some(false),
        rainy_capabilities_v2: Some(RainyCapabilitiesV2::default()),
        ..Default::default()
    };
    let anonymous = ModelCatalogItem {
        id: "model/anonymous".to_string(),
        rainy_model_tier: Some(ModelTier::Premium),
        rainy_capabilities_v2: Some(RainyCapabilitiesV2::default()),
        ..Default::default()
    };

    let selected = select_models(
        &[accessible, privacy_blocked, anonymous],
        &ModelSelectionCriteria {
            allowed_tiers: vec![ModelTier::Premium],
            require_organization_access: true,
            ..Default::default()
        },
    );

    assert_eq!(selected.len(), 1);
    assert_eq!(selected[0].id, "model/accessible");
}

#[test]
fn test_select_models_ranks_by_prompt_completion_and_context() {
    let cheap = ModelCatalogItem {
        id: "model/cheap".to_string(),
        context_length: Some(128000),
        pricing: Some(ModelPricing {
            prompt: Some("0.000001".to_string()),
            completion: Some("0.000002".to_string()),
        }),
        rainy_capabilities_v2: Some(RainyCapabilitiesV2 {
            multimodal: rainy_sdk::RainyMultimodalCapabilitiesV2 {
                input: vec!["text".to_string()],
                output: vec!["text".to_string()],
            },
            reasoning: rainy_sdk::RainyReasoningCapabilitiesV2 {
                supported: true,
                controls: Some(rainy_sdk::ReasoningControls {
                    reasoning_effort: Some(true),
                    effort: Some(vec![
                        "low".to_string(),
                        "medium".to_string(),
                        "high".to_string(),
                    ]),
                    ..Default::default()
                }),
                profiles: vec![rainy_sdk::ReasoningProfile {
                    provider: rainy_sdk::ReasoningProvider::Other,
                    parameter_path: "reasoning.effort".to_string(),
                    values: Some(vec![
                        "low".to_string(),
                        "medium".to_string(),
                        "high".to_string(),
                    ]),
                    notes: None,
                }],
                ..Default::default()
            },
            parameters: rainy_sdk::RainyParametersCapabilitiesV2 {
                accepted: vec![
                    "tools".to_string(),
                    "response_format".to_string(),
                    "reasoning".to_string(),
                ],
            },
        }),
        ..Default::default()
    };
    let expensive = ModelCatalogItem {
        id: "model/expensive".to_string(),
        context_length: Some(1_000_000),
        pricing: Some(ModelPricing {
            prompt: Some("0.00001".to_string()),
            completion: Some("0.00002".to_string()),
        }),
        rainy_capabilities_v2: cheap.rainy_capabilities_v2.clone(),
        ..Default::default()
    };

    let selected = select_models(
        &[expensive, cheap.clone()],
        &ModelSelectionCriteria {
            require_tools: Some(true),
            require_structured_output: Some(true),
            reasoning_mode: Some(ReasoningMode::Effort),
            reasoning_value: Some("high".to_string()),
            ..Default::default()
        },
    );

    assert_eq!(selected.len(), 2);
    assert_eq!(selected[0].id, cheap.id);
}

#[test]
fn test_build_reasoning_config_by_profile() {
    let model = ModelCatalogItem {
        id: "minimax/minimax-m2.5".to_string(),
        rainy_capabilities_v2: Some(serde_json::from_value(serde_json::json!({
            "multimodal": { "input": ["text"], "output": ["text"] },
            "reasoning": {
                "supported": true,
                "controls": {
                    "reasoning_effort": true,
                    "effort": ["low", "medium", "high"],
                    "thinking_budget": { "min": 0, "max": 32768, "disable_value": 0 }
                },
                "profiles": [
                    { "provider": "other", "parameter_path": "reasoning.effort", "values": ["low", "medium", "high"] },
                    { "provider": "other", "parameter_path": "thinking_config.thinking_budget" }
                ]
            },
            "parameters": { "accepted": ["reasoning"] }
        })).expect("caps deserialize")),
        ..Default::default()
    };

    let effort_payload = build_reasoning_config(
        &model,
        &ReasoningPreference {
            mode: ReasoningMode::Effort,
            value: Some("high".to_string()),
            budget: None,
        },
    )
    .expect("effort payload");
    assert_eq!(effort_payload["reasoning"]["effort"], "high");

    let budget_payload = build_reasoning_config(
        &model,
        &ReasoningPreference {
            mode: ReasoningMode::ThinkingBudget,
            value: None,
            budget: Some(1024),
        },
    )
    .expect("budget payload");
    assert_eq!(budget_payload["thinking_config"]["thinking_budget"], 1024);
}

#[test]
fn test_build_reasoning_config_returns_none_without_confirmed_profile() {
    let model = ModelCatalogItem {
        id: "google/gemini-3.1-pro-preview".to_string(),
        rainy_capabilities_v2: Some(
            serde_json::from_value(serde_json::json!({
                "multimodal": { "input": ["text"], "output": ["text"] },
                "reasoning": {
                    "supported": true,
                    "controls": {
                        "reasoning_toggle": true,
                        "observed_parameters": ["reasoning", "include_reasoning"]
                    },
                    "profiles": []
                },
                "parameters": { "accepted": ["reasoning", "include_reasoning"] }
            }))
            .expect("caps deserialize"),
        ),
        ..Default::default()
    };

    let payload = build_reasoning_config(
        &model,
        &ReasoningPreference {
            mode: ReasoningMode::Effort,
            value: Some("high".to_string()),
            budget: None,
        },
    );

    assert!(payload.is_none());
}
