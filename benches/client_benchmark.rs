use criterion::{Criterion, criterion_group, criterion_main};
use rainy_sdk::{
    AuthConfig, ChatCompletionRequest, ChatMessage, ChatStreamEvent, RainyBillingStreamEvent,
    RainyClient,
};
use std::hint::black_box;

fn valid_api_key() -> String {
    format!("ra-{}", "a".repeat(48))
}

fn sample_chunk() -> serde_json::Value {
    serde_json::json!({
        "id": "chatcmpl_benchmark",
        "object": "chat.completion.chunk",
        "created": 1_741_171_200u64,
        "model": "gpt-5",
        "choices": [{
            "index": 0,
            "delta": {"role": "assistant", "content": "hello"},
            "finish_reason": null
        }]
    })
}

fn sample_billing() -> serde_json::Value {
    serde_json::to_value(RainyBillingStreamEvent {
        plan_id: Some("payg".to_string()),
        charged_credits: Some(0.12345),
        usage: None,
    })
    .expect("billing serializes")
}

fn bench_client_construction(c: &mut Criterion) {
    c.bench_function("client_construction", |b| {
        b.iter(|| {
            let config = AuthConfig::new(black_box(valid_api_key()))
                .with_retry(false)
                .with_timeout(30);
            RainyClient::with_config(config).expect("benchmark client")
        })
    });
}

fn bench_request_serialization(c: &mut Criterion) {
    let request = ChatCompletionRequest::new(
        "gpt-5",
        vec![ChatMessage::user("benchmark request serialization")],
    )
    .with_temperature(0.2)
    .with_max_completion_tokens(128);

    c.bench_function("chat_request_serialization", |b| {
        b.iter(|| serde_json::to_vec(black_box(&request)).expect("request serializes"))
    });
}

fn bench_stream_event_parsing(c: &mut Criterion) {
    let chunk = sample_chunk();
    let billing = sample_billing();

    c.bench_function("sse_chunk_event_parsing", |b| {
        b.iter(|| ChatStreamEvent::from_sse_event(Some("message"), black_box(chunk.clone())))
    });

    c.bench_function("sse_billing_event_parsing", |b| {
        b.iter(|| {
            ChatStreamEvent::from_sse_event(Some("rainy.billing"), black_box(billing.clone()))
        })
    });
}

criterion_group!(
    benches,
    bench_client_construction,
    bench_request_serialization,
    bench_stream_event_parsing
);
criterion_main!(benches);
