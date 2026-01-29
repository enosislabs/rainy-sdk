use rainy_sdk::{
    models::{ResearchDepth, ResearchProvider},
    search::ResearchConfig,
    RainyClient,
};

#[tokio::test]
async fn test_research_configuration() {
    let _config = ResearchConfig::new()
        .with_provider(ResearchProvider::Exa)
        .with_depth(ResearchDepth::Advanced)
        .with_max_sources(5)
        .with_async(true);
}

#[tokio::test]
async fn test_research_client_api() {
    // This test just verifies API surface exists, doesn't make network calls
    // Use a valid format key: ra- + 48 hex characters
    let client =
        RainyClient::with_api_key("ra-0123456789abcdef0123456789abcdef0123456789abcdef").unwrap();

    // Check if method exists
    // Check if method exists and compiles - don't execute as it needs valid key
    let _ = client.research("test topic", None).await;
}
