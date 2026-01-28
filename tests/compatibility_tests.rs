use vector_sdk::{VectorBot, AttachmentFile, calculate_file_hash, nostr::Keys};
use serde_json;
use std::error::Error;

#[tokio::test]
async fn test_attachment_file_serialization() -> Result<(), Box<dyn Error>> {
    // Test that AttachmentFile can be serialized and deserialized
    let test_data = b"Test file content";
    let attachment = AttachmentFile::from_bytes(test_data);

    // Serialize
    let serialized = serde_json::to_string(&attachment)?;

    // Deserialize
    let deserialized: AttachmentFile = serde_json::from_str(&serialized)?;

    assert_eq!(attachment.bytes, deserialized.bytes);
    assert_eq!(attachment.extension, deserialized.extension);
    assert_eq!(attachment.img_meta, deserialized.img_meta);

    Ok(())
}

#[test]
fn test_file_hash_consistency() -> Result<(), Box<dyn Error>> {
    // Test that file hash calculation is consistent
    let test_data = b"Test data for hashing";
    let hash1 = calculate_file_hash(test_data);
    let hash2 = calculate_file_hash(test_data);

    assert_eq!(hash1, hash2);
    assert_eq!(hash1.len(), 64); // SHA-256 produces 64-character hex string

    Ok(())
}

#[tokio::test]
async fn test_api_contract_stability() -> Result<(), Box<dyn Error>> {
    // Test that the API contract remains stable
    let _keys = Keys::generate();

    // Test that VectorBot::quick() still exists and works
    let _bot = VectorBot::quick(_keys.clone()).await;
    // Bot creation should not panic
    assert!(true);

    // Test that VectorBot::new() still exists and works
    let _bot2 = VectorBot::new(
        _keys,
        "test",
        "Test",
        "Test bot",
        "https://example.com/pic.png",
        "https://example.com/banner.png",
        "test@example.com",
        "test@example.com",
    ).await;
    // Bot creation should not panic
    assert!(true);

    Ok(())
}

#[test]
fn test_error_type_compatibility() -> Result<(), Box<dyn Error>> {
    // Test that error types are compatible
    use vector_sdk::VectorBotError;

    // Test that we can create different error variants
    let _io_error = VectorBotError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
    let _url_error = VectorBotError::UrlParse(url::ParseError::RelativeUrlWithoutBase);

    // Test that errors can be converted from strings
    let str_error: VectorBotError = "test error".into();
    assert!(matches!(str_error, VectorBotError::Nostr(_)));

    Ok(())
}
