use futures::StreamExt;
use rainy_sdk::{
    AnthropicMessage, AnthropicMessageRequest, AnthropicMessageStreamEventType, AuthConfig,
    ChatCompletionRequest, ChatMessage, DEFAULT_ANTHROPIC_VERSION, EmbeddingsRequest, RainyClient,
    RainyError, ResponsesEventType, ResponsesRequest, RetryConfig,
};

fn standard_key() -> String {
    format!("ra-{}", "a".repeat(48))
}

fn client_for(server: &mockito::ServerGuard) -> RainyClient {
    RainyClient::with_config(
        AuthConfig::new(standard_key())
            .with_base_url(server.url())
            .with_timeout(5),
    )
    .expect("client")
}

fn maybe_server() -> Option<mockito::ServerGuard> {
    match std::panic::catch_unwind(mockito::Server::new) {
        Ok(server) => Some(server),
        Err(_) => {
            eprintln!("Skipping transport_contract_test: mock server unavailable");
            None
        }
    }
}

#[tokio::test]
async fn messages_request_uses_anthropic_headers_and_typed_response() {
    let Some(mut server) = maybe_server() else {
        return;
    };

    let request = AnthropicMessageRequest::new(
        "anthropic/claude-sonnet",
        vec![AnthropicMessage::user("hello")],
        128,
    )
    .with_system("Be concise");
    let authorization = format!("Bearer {}", standard_key());

    let mock = server
        .mock("POST", "/api/v1/messages")
        .match_header("authorization", authorization.as_str())
        .match_header("anthropic-version", DEFAULT_ANTHROPIC_VERSION)
        .match_header("content-type", "application/json")
        .match_body(mockito::Matcher::Json(serde_json::json!({
            "model": "anthropic/claude-sonnet",
            "messages": [{"role": "user", "content": "hello"}],
            "max_tokens": 128,
            "system": "Be concise"
        })))
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            serde_json::json!({
                "id": "msg_123",
                "type": "message",
                "role": "assistant",
                "content": [{"type": "text", "text": "hi"}],
                "model": "anthropic/claude-sonnet",
                "stop_reason": "end_turn",
                "stop_sequence": null,
                "usage": {"input_tokens": 2, "output_tokens": 1}
            })
            .to_string(),
        )
        .create();

    let response = client_for(&server)
        .create_message(request)
        .await
        .expect("Messages response");

    assert_eq!(response.id, "msg_123");
    assert_eq!(
        response.content[0],
        rainy_sdk::AnthropicContentBlock::Text {
            text: "hi".to_string()
        }
    );
    mock.assert();
}

#[tokio::test]
async fn messages_stream_preserves_native_event_names_and_done() {
    let Some(mut server) = maybe_server() else {
        return;
    };

    let mock = server
        .mock("POST", "/api/v1/messages")
        .match_header("anthropic-version", DEFAULT_ANTHROPIC_VERSION)
        .match_header("accept", "text/event-stream")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(
            "event: message_start\r\ndata: {\"type\":\"message_start\"}\r\n\r\n\
             event: content_block_delta\r\ndata: {\"type\":\"content_block_delta\",\"delta\":{\"text\":\"hi\"}}\r\n\r\n\
             event: message_stop\r\ndata: {\"type\":\"message_stop\"}\r\n\r\n\
             data: [DONE]\n\n",
        )
        .create();

    let request = AnthropicMessageRequest::new(
        "anthropic/claude-sonnet",
        vec![AnthropicMessage::user("hello")],
        128,
    );
    let mut stream = client_for(&server)
        .create_message_stream(request)
        .await
        .expect("Messages stream");

    let mut kinds = Vec::new();
    while let Some(event) = stream.next().await {
        kinds.push(event.expect("valid Messages event").kind);
    }

    assert_eq!(
        kinds,
        vec![
            AnthropicMessageStreamEventType::MessageStart,
            AnthropicMessageStreamEventType::ContentBlockDelta,
            AnthropicMessageStreamEventType::MessageStop,
        ]
    );
    mock.assert();
}

#[tokio::test]
async fn responses_stream_classifies_native_events_and_omits_done_marker() {
    let Some(mut server) = maybe_server() else {
        return;
    };

    let mock = server
        .mock("POST", "/api/v1/responses")
        .with_status(200)
        .with_header("content-type", "text/event-stream")
        .with_body(
            "event: response.created\ndata: {\"type\":\"response.created\"}\n\n\
             event: response.output_text.delta\ndata: {\"type\":\"response.output_text.delta\",\"delta\":\"hello\"}\n\n\
             event: response.completed\ndata: {\"type\":\"response.completed\"}\n\n\
             data: [DONE]\n\n",
        )
        .create();

    let request = ResponsesRequest::text("provider/model", "hello");
    let mut stream = client_for(&server)
        .create_response_stream(request)
        .await
        .expect("Responses stream");

    let mut events = Vec::new();
    while let Some(event) = stream.next().await {
        events.push(event.expect("valid Responses event"));
    }

    assert_eq!(events.len(), 3);
    assert_eq!(events[0].kind, ResponsesEventType::ResponseCreated);
    assert_eq!(events[1].kind, ResponsesEventType::OutputTextDelta);
    assert_eq!(events[1].text_delta(), Some("hello"));
    assert_eq!(events[2].kind, ResponsesEventType::ResponseCompleted);
    mock.assert();
}

#[tokio::test]
async fn readiness_probe_uses_the_root_route_and_typed_status() {
    let Some(mut server) = maybe_server() else {
        return;
    };

    let mock = server
        .mock("GET", "/ready")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"status":"ready","version":"3.7.0","timestamp":"now"}"#)
        .create();

    let readiness = client_for(&server)
        .readiness_check()
        .await
        .expect("readiness response");
    assert_eq!(readiness.status, "ready");
    assert_eq!(readiness.version.as_deref(), Some("3.7.0"));
    mock.assert();
}

#[tokio::test]
async fn inference_post_is_not_replayed_after_server_error() {
    let Some(mut server) = maybe_server() else {
        return;
    };

    let mock = server
        .mock("POST", "/api/v1/chat/completions")
        .expect(1)
        .with_status(500)
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":{"code":"UPSTREAM_ERROR","message":"failed"}}"#)
        .create();

    let client = client_for(&server).with_retry_config(RetryConfig {
        max_retries: 3,
        base_delay_ms: 0,
        max_delay_ms: 0,
        backoff_multiplier: 2.0,
        jitter: false,
    });
    let result = client
        .chat_completion(ChatCompletionRequest::new(
            "provider/model",
            vec![ChatMessage::user("hello")],
        ))
        .await;

    assert!(matches!(
        result,
        Err(RainyError::Api {
            status_code: 500,
            ..
        })
    ));
    mock.assert();
}

#[tokio::test]
async fn rate_limit_error_keeps_bounded_retry_after_header() {
    let Some(mut server) = maybe_server() else {
        return;
    };

    let mock = server
        .mock("POST", "/api/v1/chat/completions")
        .with_status(429)
        .with_header("retry-after", "7")
        .with_header("content-type", "application/json")
        .with_body(r#"{"error":{"code":"RATE_LIMIT_EXCEEDED","message":"slow down"}}"#)
        .create();

    let result = client_for(&server)
        .chat_completion(ChatCompletionRequest::new(
            "provider/model",
            vec![ChatMessage::user("hello")],
        ))
        .await;

    match result {
        Err(RainyError::RateLimit { retry_after, .. }) => assert_eq!(retry_after, Some(7)),
        other => panic!("expected rate-limit error, got {other:?}"),
    }
    mock.assert();
}

#[tokio::test]
async fn authenticated_redirect_is_not_followed_to_another_origin() {
    let Some(mut first) = maybe_server() else {
        return;
    };
    let Some(mut second) = maybe_server() else {
        return;
    };

    let first_mock = first
        .mock("GET", "/api/v1/models")
        .expect(1)
        .with_status(302)
        .with_header("location", &format!("{}/api/v1/models", second.url()))
        .create();
    let second_mock = second.mock("GET", "/api/v1/models").expect(0).create();

    let result = client_for(&first).get_available_models().await;
    assert!(matches!(
        result,
        Err(RainyError::Api {
            status_code: 302,
            ..
        })
    ));
    first_mock.assert();
    second_mock.assert();
}

#[tokio::test]
async fn oversized_error_body_is_rejected_before_unbounded_buffering() {
    let Some(mut server) = maybe_server() else {
        return;
    };

    let mock = server
        .mock("POST", "/api/v1/chat/completions")
        .with_status(500)
        .with_body("x".repeat(70 * 1024))
        .create();

    let result = client_for(&server)
        .chat_completion(ChatCompletionRequest::new(
            "provider/model",
            vec![ChatMessage::user("hello")],
        ))
        .await;

    match result {
        Err(RainyError::PayloadTooLarge { max_bytes, .. }) => {
            assert_eq!(max_bytes, 64 * 1024)
        }
        other => panic!("expected bounded-payload error, got {other:?}"),
    }
    mock.assert();
}

#[tokio::test]
async fn oversized_general_request_body_is_rejected_before_network_io() {
    let Some(mut server) = maybe_server() else {
        return;
    };

    let mock = server.mock("POST", "/api/v1/search").expect(0).create();
    let large_query = "x".repeat(1_048_576);
    let result = client_for(&server).search(large_query, None, None).await;

    assert!(matches!(
        result,
        Err(RainyError::PayloadTooLarge {
            max_bytes: 1_048_576,
            ..
        })
    ));
    mock.assert();
}

#[tokio::test]
async fn model_request_route_uses_the_larger_embedding_body_limit() {
    let Some(mut server) = maybe_server() else {
        return;
    };

    let mock = server
        .mock("POST", "/api/v1/embeddings")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(
            serde_json::json!({
                "object": "list",
                "data": [{"object": "embedding", "index": 0, "embedding": [0.25]}],
                "model": "provider/embedding",
                "usage": {"prompt_tokens": 1, "total_tokens": 1}
            })
            .to_string(),
        )
        .create();

    let request = EmbeddingsRequest::text("provider/embedding", "x".repeat(2 * 1024 * 1024));
    let result = client_for(&server).create_embeddings(request).await;

    assert!(
        result.is_ok(),
        "model-sized embedding request was rejected: {result:?}"
    );
    mock.assert();
}
