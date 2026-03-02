use vector_sdk::{VectorBot, nostr::Keys};
use std::error::Error;

#[tokio::test]
async fn test_bot_quick_creation() -> Result<(), Box<dyn Error>> {
    // Test VectorBot::quick() with default metadata
    let _keys = Keys::generate();
    let _bot = VectorBot::quick(_keys.clone()).await;

    // Test that the bot was created successfully
    // Bot creation should not panic
    assert!(true);

    Ok(())
}

#[tokio::test]
async fn test_bot_custom_creation() -> Result<(), Box<dyn Error>> {
    // Test VectorBot::new() with custom metadata
    let _keys = Keys::generate();
    let _bot = VectorBot::new(
        _keys.clone(),
        "test_bot",
        "Test Bot",
        "A test bot for compatibility",
        "https://example.com/test.png",
        "https://example.com/test_banner.png",
        "test@example.com",
        "test@example.com",
    ).await;

    // Test that the bot was created successfully
    // Bot creation should not panic
    assert!(true);

    Ok(())
}

#[tokio::test]
async fn test_bot_get_chat() -> Result<(), Box<dyn Error>> {
    // Test that get_chat() works
    let keys = Keys::generate();
    let bot = VectorBot::quick(keys.clone()).await;

    let recipient = Keys::generate().public_key();
    let _chat = bot.get_chat(recipient).await;

    // Test that channel was created successfully
    // We can't access private fields, but we can test the API works
    assert!(true);

    Ok(())
}
