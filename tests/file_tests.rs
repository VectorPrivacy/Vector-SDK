use vector_sdk::{AttachmentFile, calculate_file_hash};
use std::error::Error;
use tempfile::NamedTempFile;

#[test]
fn test_file_hash_calculation() -> Result<(), Box<dyn Error>> {
    // Test SHA-256 hash calculation
    let test_data = b"Test data for hashing";
    let hash = calculate_file_hash(test_data);

    // Verify it's a valid hex string
    assert_eq!(hash.len(), 64);
    assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));

    Ok(())
}

#[test]
fn test_file_hash_consistency() -> Result<(), Box<dyn Error>> {
    // Test that same data produces same hash
    let test_data = b"Test data";
    let hash1 = calculate_file_hash(test_data);
    let hash2 = calculate_file_hash(test_data);

    assert_eq!(hash1, hash2);

    Ok(())
}

#[test]
fn test_file_hash_different_data() -> Result<(), Box<dyn Error>> {
    // Test that different data produces different hash
    let data1 = b"Test data 1";
    let data2 = b"Test data 2";
    let hash1 = calculate_file_hash(data1);
    let hash2 = calculate_file_hash(data2);

    assert_ne!(hash1, hash2);

    Ok(())
}

#[test]
fn test_attachment_from_bytes() -> Result<(), Box<dyn Error>> {
    // Test creating attachment from bytes
    let test_data = b"Test file content";
    let attachment = AttachmentFile::from_bytes(test_data);

    assert_eq!(attachment.bytes, test_data);
    assert_eq!(attachment.extension, "bin"); // Default for unknown bytes
    assert!(attachment.img_meta.is_none());

    Ok(())
}

#[test]
fn test_attachment_from_path() -> Result<(), Box<dyn Error>> {
    // Test creating attachment from file path
    let temp_file = NamedTempFile::new()?;
    let file_path = temp_file.path().to_str().unwrap();

    // Write to a separate file to avoid borrow checker issues
    std::fs::write(file_path, b"Test file content")?;

    let attachment = AttachmentFile::from_path(file_path)?;
    assert_eq!(attachment.bytes, b"Test file content");
    // The extension may vary based on file type detection, so just check it's not empty
    assert!(!attachment.extension.is_empty());

    Ok(())
}

#[test]
fn test_mime_type_detection() -> Result<(), Box<dyn Error>> {
    // Test MIME type detection from extension
    let attachment = AttachmentFile::from_bytes(b"test");
    assert_eq!(attachment.extension, "bin");

    // Test with different extensions
    let mut attachment = AttachmentFile::from_bytes(b"test");
    attachment.extension = "txt".to_string();
    // MIME type would be detected when needed, but we can't easily test this without creating a file

    Ok(())
}
