use rainy_sdk::{
    client::RainyClient,
    error::RainyError,
    models::{
        model_constants::GOOGLE_GEMINI_3_PRO, ChatCompletionRequest, ChatMessage, ContentPart,
        EnhancedChatMessage, FunctionDefinition, ThinkingConfig, ThinkingLevel, Tool, ToolType,
    },
};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), RainyError> {
    // Initialize the client
    let client = RainyClient::with_api_key("your-api-key-here")?;

    println!("🧠 Gemini 3 Pro Thinking Capabilities Demo\n");

    // Example 1: Basic thinking with high reasoning
    println!("1. Complex reasoning task with high thinking level:");
    let request = ChatCompletionRequest::new(
        GOOGLE_GEMINI_3_PRO,
        vec![ChatMessage::user(
            "Analyze the potential economic impacts of implementing a universal basic income \
             in a developed country. Consider both short-term and long-term effects, \
             including impacts on employment, inflation, government finances, and social welfare.",
        )],
    )
    .with_thinking_config(ThinkingConfig::high_reasoning())
    .with_max_tokens(2000);

    match client.create_chat_completion(request).await {
        Ok(response) => {
            println!("Response: {}", response.choices[0].message.content);
            if let Some(usage) = &response.usage {
                println!("Tokens used: {}", usage.total_tokens);
            }
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n{}\n", "=".repeat(80));

    // Example 2: Fast response with low thinking
    println!("2. Quick task with low thinking level:");
    let request = ChatCompletionRequest::new(
        GOOGLE_GEMINI_3_PRO,
        vec![ChatMessage::user(
            "List 5 programming languages and their primary use cases.",
        )],
    )
    .with_thinking_level(ThinkingLevel::Low)
    .with_include_thoughts(false);

    match client.create_chat_completion(request).await {
        Ok(response) => {
            println!("Response: {}", response.choices[0].message.content);
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n{}\n", "=".repeat(80));

    // Example 3: Function calling with thought signatures
    println!("3. Function calling with thought signatures:");

    let tools = vec![
        Tool {
            r#type: ToolType::Function,
            function: FunctionDefinition {
                name: "get_weather".to_string(),
                description: Some("Get current weather for a location".to_string()),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "location": {
                            "type": "string",
                            "description": "The city name"
                        }
                    },
                    "required": ["location"]
                })),
            },
        },
        Tool {
            r#type: ToolType::Function,
            function: FunctionDefinition {
                name: "book_restaurant".to_string(),
                description: Some("Book a restaurant reservation".to_string()),
                parameters: Some(json!({
                    "type": "object",
                    "properties": {
                        "restaurant": {"type": "string"},
                        "time": {"type": "string"},
                        "party_size": {"type": "integer"}
                    },
                    "required": ["restaurant", "time", "party_size"]
                })),
            },
        },
    ];

    let request = ChatCompletionRequest::new(
        GOOGLE_GEMINI_3_PRO,
        vec![ChatMessage::user(
            "Check the weather in Paris and if it's nice, book a table for 2 at Le Bernardin for 7 PM tonight."
        )]
    )
    .with_thinking_config(ThinkingConfig::gemini_3(ThinkingLevel::High, true))
    .with_tools(tools);

    match client.create_chat_completion(request).await {
        Ok(response) => {
            println!("Response: {}", response.choices[0].message.content);
            // Note: In a real implementation, you would handle function calls
            // and preserve thought signatures across multiple turns
        }
        Err(e) => println!("Error: {}", e),
    }

    println!("\n{}\n", "=".repeat(80));

    // Example 4: Enhanced message format with thought signatures
    println!("4. Using enhanced message format:");

    // Simulate a conversation with thought signatures
    let enhanced_messages = vec![
        EnhancedChatMessage::user("What's the best approach to solve climate change?"),
        EnhancedChatMessage::with_parts(
            rainy_sdk::models::MessageRole::Assistant,
            vec![
                ContentPart::text("Let me think through this systematically...").as_thought(),
                ContentPart::text("Climate change requires a multi-faceted approach...")
                    .with_thought_signature("encrypted_signature_here"),
            ],
        ),
    ];

    println!(
        "Enhanced message structure created with {} parts",
        enhanced_messages[1].parts.len()
    );

    // Example 5: Model validation
    println!("\n5. Model capability validation:");

    let gemini_3_request =
        ChatCompletionRequest::new(GOOGLE_GEMINI_3_PRO, vec![ChatMessage::user("Test message")]);

    println!(
        "Gemini 3 Pro supports thinking: {}",
        gemini_3_request.supports_thinking()
    );
    println!(
        "Gemini 3 Pro requires thought signatures: {}",
        gemini_3_request.requires_thought_signatures()
    );

    // Validate configuration
    let thinking_request = gemini_3_request
        .with_thinking_level(ThinkingLevel::High)
        .with_include_thoughts(true);

    match thinking_request.validate_openai_compatibility() {
        Ok(()) => println!("✅ Configuration is valid"),
        Err(e) => println!("❌ Configuration error: {}", e),
    }

    Ok(())
}
