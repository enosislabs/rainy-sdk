use rainy_sdk::{
    model_constants::GOOGLE_GEMINI_3_PRO, ChatCompletionStreamResponse,
    OpenAIChatCompletionRequest, OpenAIChatCompletionResponse, OpenAIChatMessage,
    OpenAIContentPart, OpenAIFunctionCall, OpenAIMessageRole, OpenAIToolCall, RainyClient,
    ThinkingConfig, ThinkingLevel, Tool, ToolChoice, ToolFunction, ToolType,
};

#[test]
fn test_openai_chat_request_serialization_supports_tool_history() {
    let request = OpenAIChatCompletionRequest::new(
        GOOGLE_GEMINI_3_PRO,
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

    assert_eq!(json["model"], GOOGLE_GEMINI_3_PRO);
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
        GOOGLE_GEMINI_3_PRO,
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
        GOOGLE_GEMINI_3_PRO,
        vec![OpenAIChatMessage::user("ping")],
    );

    let _future = client.create_openai_chat_completion(request.clone());
    let _stream_future = client.create_openai_chat_completion_stream(request);

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
