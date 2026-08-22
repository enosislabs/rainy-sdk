use rainy_sdk::{
    ChatCompletionStreamResponse, ChatStreamEvent, OpenAIChatCompletionRequest,
    OpenAIChatCompletionResponse, OpenAIChatMessage, OpenAIContentPart, OpenAIFunctionCall,
    OpenAIMessageRole, OpenAIToolCall, RainyClient, ThinkingConfig, ThinkingLevel, Tool,
    ToolChoice, ToolFunction, ToolType,
};

#[test]
fn test_openai_chat_request_serialization_supports_tool_history() {
    let request = OpenAIChatCompletionRequest::new(
        "gemini-3-pro-preview",
        vec![
            OpenAIChatMessage::system("Use tools when needed."),
            OpenAIChatMessage::user("List files in the workspace."),
            OpenAIChatMessage::assistant_with_tool_calls(vec![OpenAIToolCall {
                id: "call_123".to_string(),
                r#type: "function".to_string(),
                extra_content: Some(serde_json::json!({
                    "google": { "thought_signature": "sig_abc" }
                })),
                function: OpenAIFunctionCall {
                    name: "list_files".to_string(),
                    arguments: "{\"path\":\".\"}".to_string(),
                },
            }]),
            OpenAIChatMessage::tool("call_123", "{\"entries\":[\"src\",\"Cargo.toml\"]}"),
        ],
    )
    .with_thinking_config(ThinkingConfig::gemini_3(ThinkingLevel::High, true))
    .with_tools(vec![Tool {
        r#type: ToolType::Function,
        function: rainy_sdk::FunctionDefinition {
            name: "list_files".to_string(),
            description: Some("List directory contents".to_string()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {
                    "path": { "type": "string" }
                },
                "required": ["path"]
            })),
        },
    }])
    .with_tool_choice(ToolChoice::Tool {
        r#type: ToolType::Function,
        function: ToolFunction {
            name: "list_files".to_string(),
        },
    });

    let json = serde_json::to_value(&request).expect("serialize request");

    assert_eq!(json["model"], "gemini-3-pro-preview");
    assert_eq!(json["messages"][2]["role"], "assistant");
    assert!(json["messages"][2]["content"].is_null());
    assert_eq!(json["messages"][2]["tool_calls"][0]["id"], "call_123");
    assert_eq!(
        json["messages"][2]["tool_calls"][0]["extra_content"]["google"]["thought_signature"],
        "sig_abc"
    );
    assert_eq!(json["messages"][3]["role"], "tool");
    assert_eq!(json["messages"][3]["tool_call_id"], "call_123");
    assert_eq!(json["tools"][0]["function"]["name"], "list_files");
}

#[test]
fn test_openai_chat_request_supports_multimodal_content() {
    let request = OpenAIChatCompletionRequest::new(
        "gemini-3-pro-preview",
        vec![OpenAIChatMessage::user(
            rainy_sdk::OpenAIMessageContent::parts(vec![
                OpenAIContentPart::text("Describe this image."),
                OpenAIContentPart::image_url_with_detail("https://example.com/image.png", "high"),
            ]),
        )],
    );

    let json = serde_json::to_value(&request).expect("serialize multimodal request");
    assert_eq!(json["messages"][0]["content"][0]["type"], "text");
    assert_eq!(json["messages"][0]["content"][1]["type"], "image_url");
    assert_eq!(
        json["messages"][0]["content"][1]["image_url"]["detail"],
        "high"
    );
}

#[test]
fn test_openai_chat_request_serialization_supports_modern_fields() {
    let mut metadata = std::collections::HashMap::new();
    metadata.insert("flow".to_string(), "sdk-test".to_string());

    let request = OpenAIChatCompletionRequest::new("gpt-5", vec![OpenAIChatMessage::user("hello")])
        .with_max_completion_tokens(256)
        .with_stream_options(serde_json::json!({ "include_usage": true }))
        .with_reasoning(serde_json::json!({ "effort": "medium" }))
        .with_include_reasoning(true)
        .with_service_tier("auto")
        .with_metadata(metadata);

    let json = serde_json::to_value(&request).expect("serialize modern openai fields");
    assert_eq!(json["max_completion_tokens"], 256);
    assert_eq!(json["stream_options"]["include_usage"], true);
    assert_eq!(json["reasoning"]["effort"], "medium");
    assert_eq!(json["include_reasoning"], true);
    assert_eq!(json["service_tier"], "auto");
    assert_eq!(json["metadata"]["flow"], "sdk-test");
}

#[test]
fn test_openai_chat_response_deserializes_tool_calls() {
    let payload = serde_json::json!({
        "id": "chatcmpl_123",
        "object": "chat.completion",
        "created": 1741171200u64,
        "model": "gemini-3-pro-preview",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_123",
                    "type": "function",
                    "extra_content": {
                        "google": { "thought_signature": "sig_abc" }
                    },
                    "function": {
                        "name": "list_files",
                        "arguments": "{\"path\":\".\"}"
                    }
                }]
            },
            "finish_reason": "tool_calls"
        }],
        "usage": {
            "prompt_tokens": 10,
            "completion_tokens": 5,
            "total_tokens": 15
        }
    });

    let response: OpenAIChatCompletionResponse =
        serde_json::from_value(payload).expect("deserialize response");

    assert_eq!(
        response.choices[0].message.role,
        OpenAIMessageRole::Assistant
    );
    assert!(response.choices[0].message.content.is_none());
    assert_eq!(
        response.choices[0].message.tool_calls.as_ref().unwrap()[0]
            .function
            .name,
        "list_files"
    );
}

#[test]
fn test_openai_chat_stream_surface_exists() {
    let client = RainyClient::with_api_key(format!("ra-{}", "d".repeat(48)))
        .expect("failed to build client");

    let request = OpenAIChatCompletionRequest::new(
        "gemini-3-pro-preview",
        vec![OpenAIChatMessage::user("ping")],
    );

    let _future = client.create_openai_chat_completion(request.clone());
    let _stream_future = client.create_openai_chat_completion_stream(request);
    let _stream_events_future =
        client.create_openai_chat_completion_stream_events(OpenAIChatCompletionRequest::new(
            "gemini-3-pro-preview",
            vec![OpenAIChatMessage::user("ping")],
        ));

    let chunk_payload = serde_json::json!({
        "id": "chatcmpl_chunk_1",
        "object": "chat.completion.chunk",
        "created": 1741171200u64,
        "model": "gemini-3-pro-preview",
        "choices": [{
            "index": 0,
            "delta": {
                "role": "assistant",
                "tool_calls": [{
                    "index": 0,
                    "id": "call_123",
                    "type": "function",
                    "function": {
                        "name": "list_files",
                        "arguments": "{\"path\":\".\"}"
                    }
                }]
            },
            "finish_reason": null
        }]
    });

    let chunk: ChatCompletionStreamResponse =
        serde_json::from_value(chunk_payload).expect("deserialize stream chunk");
    assert_eq!(
        chunk.choices[0].delta.tool_calls.as_ref().unwrap()[0]
            .function
            .as_ref()
            .unwrap()
            .name
            .as_deref(),
        Some("list_files")
    );
}

#[test]
fn test_chat_stream_event_parsing_chunk_and_billing() {
    let chunk_payload = serde_json::json!({
        "id": "chatcmpl_chunk_1",
        "object": "chat.completion.chunk",
        "created": 1741171200u64,
        "model": "gpt-5",
        "choices": [{
            "index": 0,
            "delta": { "role": "assistant", "content": "hi" },
            "finish_reason": null
        }]
    });

    let billing_payload = serde_json::json!({
        "plan_id": "payg",
        "charged_credits": 0.12345,
        "usage": {
            "prompt_tokens": 12,
            "completion_tokens": 7
        }
    });

    let chunk_event = ChatStreamEvent::from_value(chunk_payload);
    let billing_event = ChatStreamEvent::from_value(billing_payload);

    match chunk_event {
        ChatStreamEvent::Chunk(chunk) => {
            assert_eq!(chunk.model, "gpt-5");
        }
        other => panic!("expected chunk event, got {other:?}"),
    }

    match billing_event {
        ChatStreamEvent::Billing(event) => {
            assert_eq!(event.plan_id.as_deref(), Some("payg"));
            assert_eq!(event.usage.as_ref().and_then(|u| u.prompt_tokens), Some(12));
        }
        other => panic!("expected billing event, got {other:?}"),
    }
}

#[test]
fn test_chat_stream_event_parsing_honors_native_billing_event_name() {
    let billing_payload = serde_json::json!({
        "usage": {
            "prompt_tokens": 12,
            "completion_tokens": 7
        }
    });

    let event = ChatStreamEvent::from_sse_event(Some("rainy.billing"), billing_payload);

    match event {
        ChatStreamEvent::Billing(billing) => {
            assert_eq!(billing.plan_id, None);
            assert_eq!(billing.usage.unwrap().prompt_tokens, Some(12));
        }
        other => panic!("expected native billing event, got {other:?}"),
    }
}
