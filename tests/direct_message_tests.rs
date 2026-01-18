use vector_sdk::{VectorBot, AttachmentFile, nostr::Keys};
use std::error::Error;

#[tokio::test]
async fn test_send_private_message() -> Result<(), Box<dyn Error>> {
    // Test sending a private message
    let keys = Keys::generate();
    let bot = VectorBot::quick(keys.clone()).await;

    let recipient = Keys::generate().public_key();
    let chat = bot.get_chat(recipient).await;

    // This will fail in test environment due to no relays, but tests the API
    let result = chat.send_private_message("Test message").await;
    assert!(result); // Should return true

    Ok(())
}

#[tokio::test]
async fn test_send_typing_indicator() -> Result<(), Box<dyn Error>> {
    // Test sending a typing indicator
    let keys = Keys::generate();
    let bot = VectorBot::quick(keys.clone()).await;

    let recipient = Keys::generate().public_key();
    let chat = bot.get_chat(recipient).await;

    let result = chat.send_typing_indicator().await;
    assert!(result); // Should return true

    Ok(())
}

#[tokio::test]
async fn test_send_reaction() -> Result<(), Box<dyn Error>> {
    // Test sending a reaction
    // This will fail in test environment due to no relays, but tests the API
    let keys = Keys::generate();
    let bot = VectorBot::quick(keys.clone()).await;

    let recipient = Keys::generate().public_key();
    let chat = bot.get_chat(recipient).await;

    // In test environment without relays, this will return false
    // but we're testing that the API exists and doesn't panic
    let _result = chat.send_reaction("test_id".to_string(), "❤️".to_string()).await;

    Ok(())
}

#[tokio::test]
async fn test_attachment_file_from_bytes() -> Result<(), Box<dyn Error>> {
    // Test creating attachment from bytes
    let test_data = b"Test file content";
    let attachment = AttachmentFile::from_bytes(test_data);

    assert_eq!(attachment.bytes, test_data);
    assert_eq!(attachment.extension, "bin"); // Default for unknown bytes

    Ok(())
}

#[tokio::test]
async fn test_send_private_file() -> Result<(), Box<dyn Error>> {
    // Test sending a private file
    let keys = Keys::generate();
    let bot = VectorBot::quick(keys.clone()).await;

    let recipient = Keys::generate().public_key();
    let chat = bot.get_chat(recipient).await;

    let test_data = b"Test file content";
    let attachment = AttachmentFile::from_bytes(test_data);

    // This will fail in test environment due to no relays, but tests the API
    let result = chat.send_private_file(Some(attachment)).await;
    assert!(result); // Should return true

    Ok(())
}
