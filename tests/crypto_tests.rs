use vector_sdk::crypto::{generate_encryption_params, encrypt_data};
use std::error::Error;

#[test]
fn test_encryption_params_generation() -> Result<(), Box<dyn Error>> {
    // Test that encryption parameters can be generated
    let params = generate_encryption_params()?;
    assert!(!params.key.is_empty());
    assert!(!params.nonce.is_empty());
    Ok(())
}

#[test]
fn test_encryption_roundtrip() -> Result<(), Box<dyn Error>> {
    // Test that data can be encrypted
    let test_data = b"Test data for encryption";
    let params = generate_encryption_params()?;

    // Encrypt the data
    let encrypted = encrypt_data(test_data, &params)?;

    // Verify encryption worked
    assert_ne!(encrypted, test_data);
    assert!(!encrypted.is_empty());

    Ok(())
}

#[test]
fn test_encryption_with_different_keys() -> Result<(), Box<dyn Error>> {
    // Test that different keys produce different encrypted data
    let test_data = b"Test data";
    let params1 = generate_encryption_params()?;
    let params2 = generate_encryption_params()?;

    let encrypted1 = encrypt_data(test_data, &params1)?;
    let encrypted2 = encrypt_data(test_data, &params2)?;

    assert_ne!(encrypted1, encrypted2);
    Ok(())
}
