use vector_sdk::VectorBotError;
use std::error::Error;

#[test]
fn test_error_variants() -> Result<(), Box<dyn Error>> {
    // Test that all error variants can be created
    let _io_error = VectorBotError::Io(std::io::Error::new(std::io::ErrorKind::Other, "test"));
    let _url_error = VectorBotError::UrlParse(url::ParseError::RelativeUrlWithoutBase);
    let _serde_error = VectorBotError::SerdeJson(serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::Other, "test")));
    let _invalid_input = VectorBotError::InvalidInput("test".to_string());
    let _network_error = VectorBotError::Network("test".to_string());
    let _storage_error = VectorBotError::Storage("test".to_string());

    // Test that errors can be converted from strings
    let str_error: VectorBotError = "test error".into();
    assert!(matches!(str_error, VectorBotError::Nostr(_)));

    let str_slice_error: VectorBotError = "test slice error".into();
    assert!(matches!(str_slice_error, VectorBotError::Nostr(_)));

    Ok(())
}

#[test]
fn test_error_display() -> Result<(), Box<dyn Error>> {
    // Test that errors can be displayed
    let error = VectorBotError::InvalidInput("test error".to_string());
    let display = format!("{}", error);
    assert!(display.contains("test error"));

    Ok(())
}

#[test]
fn test_error_from_conversions() -> Result<(), Box<dyn Error>> {
    // Test From trait implementations for error conversions
    let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
    let vector_error: VectorBotError = io_error.into();
    assert!(matches!(vector_error, VectorBotError::Io(_)));

    let parse_error = url::ParseError::RelativeUrlWithoutBase;
    let vector_error: VectorBotError = parse_error.into();
    assert!(matches!(vector_error, VectorBotError::UrlParse(_)));

    Ok(())
}

#[test]
fn test_error_debug() -> Result<(), Box<dyn Error>> {
    // Test that errors can be debug printed
    let error = VectorBotError::Network("connection failed".to_string());
    let debug_str = format!("{:?}", error);
    assert!(debug_str.contains("Network"));

    Ok(())
}
