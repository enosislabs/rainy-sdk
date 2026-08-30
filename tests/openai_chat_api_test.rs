use rainy_sdk::{
    ChatCompletionStreamResponse, ChatStreamEvent, FunctionDefinition, OpenAIChatCompletionRequest,
    OpenAIChatCompletionResponse, OpenAIChatMessage, OpenAIContentPart, OpenAIFunctionCall,
    OpenAIMessageContent, OpenAIMessageRole, OpenAIToolCall, ReasoningConfig, ReasoningEffort,
    Tool, ToolChoice, ToolFunction, ToolType,
};

#[test]
fn openai_chat_serializes_tool_history_and_reasoning() {
    let request = OpenAIChatCompletionRequest::new(
        "any/chat-model",
        vec![
            OpenAIChatMessage::system("Use tools when needed."),
            OpenAIChatMessage::user("List files."),
            OpenAIChatMessage::assistant_with_tool_calls(vec![OpenAIToolCall {
                id: "call_123".to_string(),
                r#type: "function".to_string(),
                extra_content: Some(serde_json::json!({"trace": "opaque"})),
                function: OpenAIFunctionCall {
                    name: "list_files".to_string(),
                    arguments: "{\"path\":\".\"}".to_string(),
                },
            }]),
            OpenAIChatMessage::tool("call_123", "{\"entries\":[]}"),
        ],
    )
    .with_reasoning(ReasoningConfig::effort(ReasoningEffort::High))
    .with_reasoning_effort(ReasoningEffort::High)
    .with_tools(vec![Tool {
        r#type: ToolType::Function,
        function: FunctionDefinition {
            name: "list_files".to_string(),
            description: Some("List directory contents".to_string()),
            parameters: Some(serde_json::json!({
                "type": "object",
                "properties": {"path": {"type": "string"}}
            })),
        },
    }])
    .with_tool_choice(ToolChoice::Tool {
        r#type: ToolType::Function,
        function: ToolFunction {
            name: "list_files".to_string(),
        },
    });

    let json = serde_json::to_value(request).expect("serialize request");
    assert_eq!(json["model"], "any/chat-model");
    assert_eq!(json["messages"][2]["role"], "assistant");
    assert!(json["messages"][2]["content"].is_null());
    assert_eq!(json["messages"][2]["tool_calls"][0]["id"], "call_123");
    assert_eq!(json["messages"][3]["tool_call_id"], "call_123");
    assert_eq!(json["tools"][0]["function"]["name"], "list_files");
    assert_eq!(json["reasoning"]["effort"], "high");
    assert_eq!(json["reasoning_effort"], "high");
}

#[test]
fn openai_chat_supports_multimodal_parts() {
    let request = OpenAIChatCompletionRequest::new(
        "any/multimodal-model",
        vec![OpenAIChatMessage::user(OpenAIMessageContent::parts(vec![
            OpenAIContentPart::text("Describe this image."),
            OpenAIContentPart::image_url_with_detail("https://example.com/image.png", "high"),
            OpenAIContentPart::input_audio("ZmFrZQ==", "wav"),
        ]))],
    );
    let json = serde_json::to_value(request).expect("serialize multimodal request");
    assert_eq!(json["messages"][0]["content"][0]["type"], "text");
    assert_eq!(json["messages"][0]["content"][1]["type"], "image_url");
    assert_eq!(json["messages"][0]["content"][2]["type"], "input_audio");
}

#[test]
fn openai_response_deserializes_tool_calls() {
    let payload = serde_json::json!({
        "id": "chatcmpl_123",
        "object": "chat.completion",
        "created": 1_741_171_200u64,
        "model": "any/chat-model",
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_123",
                    "type": "function",
                    "extra_content": {"trace": "opaque"},
                    "function": {"name": "list_files", "arguments": "{}"}
                }]
            },
            "finish_reason": "tool_calls"
        }]
    });
    let response: OpenAIChatCompletionResponse = serde_json::from_value(payload).unwrap();
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
fn chat_stream_chunk_and_native_billing_are_typed() {
    let chunk_payload = serde_json::json!({
        "id": "chunk_1",
        "object": "chat.completion.chunk",
        "created": 1_741_171_200u64,
        "model": "any/chat-model",
        "choices": [{
            "index": 0,
            "delta": {"role": "assistant", "content": "hi"},
            "finish_reason": null
        }]
    });
    let chunk = ChatStreamEvent::from_value(chunk_payload);
    assert!(matches!(chunk, ChatStreamEvent::Chunk(_)));

    let billing = ChatStreamEvent::from_sse_event(
        Some("rainy.billing"),
        serde_json::json!({"usage": {"prompt_tokens": 12}}),
    );
    match billing {
        ChatStreamEvent::Billing(value) => {
            assert_eq!(value.usage.unwrap().prompt_tokens, Some(12));
        }
        other => panic!("expected billing event, got {other:?}"),
    }
}

#[test]
fn named_unknown_chat_events_are_not_reinterpreted_as_chunks() {
    let value = serde_json::json!({
        "id": "chunk_1",
        "object": "chat.completion.chunk",
        "created": 1,
        "model": "any/model",
        "choices": []
    });
    assert!(matches!(
        ChatStreamEvent::from_sse_event(Some("future.event"), value),
        ChatStreamEvent::Unknown { event, .. } if event == "future.event"
    ));
}

#[test]
fn chat_stream_chunk_deserializes_reasoning_and_tool_deltas() {
    let chunk: ChatCompletionStreamResponse = serde_json::from_value(serde_json::json!({
        "id": "chunk_1",
        "object": "chat.completion.chunk",
        "created": 1,
        "model": "any/model",
        "choices": [{
            "index": 0,
            "delta": {
                "role": "assistant",
                "reasoning_content": "think",
                "tool_calls": [{
                    "index": 0,
                    "id": "call_1",
                    "type": "function",
                    "function": {"name": "lookup", "arguments": "{}"}
                }]
            },
            "finish_reason": null
        }]
    }))
    .unwrap();
    assert_eq!(
        chunk.choices[0].delta.reasoning_content.as_deref(),
        Some("think")
    );
    assert_eq!(
        chunk.choices[0].delta.tool_calls.as_ref().unwrap()[0]
            .function
            .as_ref()
            .unwrap()
            .name
            .as_deref(),
        Some("lookup")
    );
}
