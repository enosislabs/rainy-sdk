use futures::StreamExt;
use rainy_sdk::{
    AnthropicContentBlock, AnthropicMessage, AnthropicMessageRequest, AnthropicTool, AuthConfig,
    ChatCompletionRequest, ChatMessage, ChatStreamEvent, RainyClient, RainyError, ResponsesRequest,
};

#[tokio::test]
async fn openai_chat_uses_bearer_and_does_not_discover_first() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/api/v1/chat/completions")
        .match_header("authorization", "Bearer protocol-secret")
        .match_header("x-api-key", mockito::Matcher::Missing)
        .match_body(mockito::Matcher::PartialJson(serde_json::json!({
            "model": "custom/chat-model",
            "messages": [{"role": "user", "content": "hello"}],
            "reasoning_effort": "high"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
              "id": "chat_1",
              "object": "chat.completion",
              "created": 1,
              "model": "custom/chat-model",
              "choices": [{"index": 0, "message": {"role": "assistant", "content": "hello back"}, "finish_reason": "stop"}]
            }"#,
        )
        .create_async()
        .await;

    let client = RainyClient::with_config(
        AuthConfig::new("protocol-secret")
            .with_base_url(server.url())
            .with_retry(false),
    )
    .expect("client");
    let response = client
        .create_chat_completion(
            ChatCompletionRequest::new("custom/chat-model", vec![ChatMessage::user("hello")])
                .with_reasoning_effort("high"),
        )
        .await
        .expect("chat response");

    assert_eq!(response.choices[0].message.content, "hello back");
    mock.assert();
}

#[tokio::test]
async fn responses_uses_the_same_bearer_protocol_boundary() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/api/v1/responses")
        .match_header("authorization", "Bearer responses-secret")
        .match_header("x-api-key", mockito::Matcher::Missing)
        .match_body(mockito::Matcher::PartialJson(serde_json::json!({
            "model": "custom/responses-model",
            "input": "hello",
            "reasoning": {"max_tokens": 2048}
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"id":"resp_1","output_text":"done"}"#)
        .create_async()
        .await;

    let client = RainyClient::with_config(
        AuthConfig::new("responses-secret")
            .with_base_url(server.url())
            .with_retry(false),
    )
    .expect("client");
    let response = client
        .create_response(
            ResponsesRequest::text("custom/responses-model", "hello").with_reasoning_budget(2048),
        )
        .await
        .expect("Responses response")
        .0;

    assert_eq!(response.text().as_deref(), Some("done"));
    mock.assert();
}

#[tokio::test]
async fn anthropic_messages_use_native_headers_and_thinking() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/api/v1/messages")
        .match_header("x-api-key", "anthropic-secret")
        .match_header("anthropic-version", "2023-06-01")
        .match_header("authorization", mockito::Matcher::Missing)
        .match_body(mockito::Matcher::PartialJson(serde_json::json!({
            "model": "custom/messages-model",
            "max_tokens": 1024,
            "system": "Be concise",
            "thinking": {"type": "enabled", "budget_tokens": 256},
            "tools": [{"name": "lookup", "input_schema": {"type": "object"}}]
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            r#"{
              "id":"msg_1",
              "type":"message",
              "role":"assistant",
              "content":[{"type":"text","text":"done"}],
              "model":"custom/messages-model",
              "stop_reason":"end_turn",
              "usage":{"input_tokens":4,"output_tokens":2}
            }"#,
        )
        .create_async()
        .await;

    let client = RainyClient::with_config(
        AuthConfig::new("anthropic-secret")
            .with_base_url(server.url())
            .with_retry(false),
    )
    .expect("client");
    let response = client
        .create_message(
            AnthropicMessageRequest::new(
                "custom/messages-model",
                vec![AnthropicMessage::user("hello")],
                1024,
            )
            .with_system("Be concise")
            .with_thinking(256)
            .with_tools(vec![AnthropicTool::new(
                "lookup",
                serde_json::json!({"type": "object"}),
            )]),
        )
        .await
        .expect("Messages response");

    assert_eq!(response.text(), "done");
    mock.assert();
}

#[tokio::test]
async fn anthropic_message_stream_preserves_native_event_names_and_types() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/api/v1/messages")
        .match_header("x-api-key", "anthropic-stream-secret")
        .match_header("anthropic-version", "2023-06-01")
        .match_header("authorization", mockito::Matcher::Missing)
        .match_body(mockito::Matcher::PartialJson(serde_json::json!({
            "model": "custom/messages-model",
            "stream": true
        })))
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(
            "event: message_start\n"
                .to_string()
                + "data: {\"type\":\"message_start\",\"message\":{\"id\":\"msg_2\"}}\n\n"
                + "event: content_block_delta\n"
                + "data: {\"type\":\"content_block_delta\",\"delta\":{\"type\":\"text_delta\",\"text\":\"hi\"}}\n\n"
                + "event: message_stop\n"
                + "data: {\"type\":\"message_stop\"}\n\n",
        )
        .create_async()
        .await;

    let client = RainyClient::with_config(
        AuthConfig::new("anthropic-stream-secret")
            .with_base_url(server.url())
            .with_retry(false),
    )
    .expect("client");
    let events = client
        .create_message_stream(AnthropicMessageRequest::new(
            "custom/messages-model",
            vec![AnthropicMessage::user("hello")],
            256,
        ))
        .await
        .expect("stream")
        .collect::<Vec<_>>()
        .await;

    assert_eq!(events.len(), 3);
    assert_eq!(
        events[0].as_ref().unwrap().event.as_deref(),
        Some("message_start")
    );
    assert_eq!(
        events[0].as_ref().unwrap().kind,
        rainy_sdk::AnthropicMessageStreamEventType::MessageStart
    );
    assert_eq!(
        events[1].as_ref().unwrap().kind,
        rainy_sdk::AnthropicMessageStreamEventType::ContentBlockDelta
    );
    assert_eq!(events[1].as_ref().unwrap().data["delta"]["text"], "hi");
    assert_eq!(
        events[2].as_ref().unwrap().kind,
        rainy_sdk::AnthropicMessageStreamEventType::MessageStop
    );
    mock.assert();
}

#[tokio::test]
async fn streaming_chat_preserves_chunks_and_native_billing_events() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/api/v1/chat/completions")
        .match_header("authorization", "Bearer stream-secret")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(
            "event: message\n"
                .to_string()
                + "data: {\"id\":\"chunk_1\",\"object\":\"chat.completion.chunk\",\"created\":1,\"model\":\"custom/model\",\"choices\":[{\"index\":0,\"delta\":{\"role\":\"assistant\",\"content\":\"hi\"},\"finish_reason\":null}]}\n\n"
                + "event: rainy.billing\n"
                + "data: {\"usage\":{\"prompt_tokens\":3}}\n\n"
                + "data: [DONE]\n\n",
        )
        .create_async()
        .await;

    let client = RainyClient::with_config(
        AuthConfig::new("stream-secret")
            .with_base_url(server.url())
            .with_retry(false),
    )
    .expect("client");
    let events = client
        .chat_completion_stream_events(ChatCompletionRequest::new(
            "custom/model",
            vec![ChatMessage::user("hello")],
        ))
        .await
        .expect("stream")
        .collect::<Vec<_>>()
        .await;

    assert_eq!(events.len(), 2);
    assert!(matches!(events[0], Ok(ChatStreamEvent::Chunk(_))));
    assert!(matches!(events[1], Ok(ChatStreamEvent::Billing(_))));
    mock.assert();
}

#[tokio::test]
async fn inference_post_is_not_replayed_on_server_error() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/api/v1/chat/completions")
        .expect(1)
        .with_status(503)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":{"code":"TEMPORARY","message":"try later"}}"#)
        .create_async()
        .await;

    let client = RainyClient::with_config(
        AuthConfig::new("retry-secret")
            .with_base_url(server.url())
            .with_max_retries(3),
    )
    .expect("client");
    let result = client
        .create_chat_completion(ChatCompletionRequest::new(
            "custom/model",
            vec![ChatMessage::user("hello")],
        ))
        .await;
    assert!(matches!(
        result,
        Err(RainyError::Api {
            status_code: 503,
            ..
        })
    ));
    mock.assert();
}

#[tokio::test]
async fn oversized_success_response_is_rejected_before_decode() {
    let mut server = mockito::Server::new_async().await;
    let mock = server
        .mock("POST", "/api/v1/responses")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(format!(
            "{{\"output_text\":\"{}\"}}",
            "x".repeat(8 * 1024 * 1024)
        ))
        .create_async()
        .await;

    let client = RainyClient::with_config(
        AuthConfig::new("size-secret")
            .with_base_url(server.url())
            .with_retry(false),
    )
    .expect("client");
    let result = client
        .create_response(ResponsesRequest::text("custom/model", "hello"))
        .await;
    assert!(matches!(result, Err(RainyError::PayloadTooLarge { .. })));
    mock.assert();
}

#[tokio::test]
async fn redirects_are_not_followed_and_error_bodies_redact_keys() {
    let mut server = mockito::Server::new_async().await;
    let secret = "error-secret";
    let mock = server
        .mock("POST", "/api/v1/responses")
        .with_status(400)
        .with_header("location", "https://example.com/redirect-target")
        .with_header("content-type", "application/json")
        .with_body(format!(
            "{{\"error\":{{\"code\":\"INVALID_REQUEST\",\"message\":\"upstream echoed {secret}\"}}}}"
        ))
        .create_async()
        .await;

    let client = RainyClient::with_config(
        AuthConfig::new(secret)
            .with_base_url(server.url())
            .with_retry(false),
    )
    .expect("client");
    let error = client
        .create_response(ResponsesRequest::text("custom/model", "hello"))
        .await
        .expect_err("request must fail");

    assert!(!error.to_string().contains(secret));
    assert!(!format!("{error:?}").contains(secret));
    mock.assert();
}

#[test]
fn public_auth_headers_are_protocol_specific_and_redacted() {
    let config = AuthConfig::new("header-secret");
    let bearer = config.build_headers().expect("Bearer headers");
    assert_eq!(bearer["authorization"], "Bearer header-secret");
    assert!(bearer.get("x-api-key").is_none());

    let anthropic = config
        .build_anthropic_headers("2023-06-01")
        .expect("Anthropic headers");
    assert_eq!(anthropic["x-api-key"], "header-secret");
    assert_eq!(anthropic["anthropic-version"], "2023-06-01");
    assert!(anthropic.get("authorization").is_none());
    assert!(!format!("{config:?}").contains("header-secret"));
    assert!(!config.to_string().contains("header-secret"));
}

#[test]
fn anthropic_tool_and_thinking_blocks_round_trip_natively() {
    let block =
        AnthropicContentBlock::tool_use("tool_1", "lookup", serde_json::json!({"key": "value"}));
    let value = serde_json::to_value(block).expect("serialize tool block");
    assert_eq!(value["type"], "tool_use");
    assert_eq!(value["name"], "lookup");
}
