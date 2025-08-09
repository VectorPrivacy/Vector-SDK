use aes::Aes256;
use aes_gcm::{AeadInPlace, AesGcm, Error as AesGcmError, KeyInit};
use generic_array::{typenum::U16, GenericArray};
use log::debug;
use rand::Rng;
use thiserror::Error;
// Removed unused import

/// Represents encryption parameters for AES-256-GCM
///
/// This struct contains the encryption key and initialization vector (nonce)
/// needed for AES-256-GCM encryption.
#[derive(Debug, Clone)]
pub struct EncryptionParams {
    /// The encryption key (hex string)
    pub key: String,
    /// The initialization vector (nonce) (hex string)
    pub nonce: String,
}

/// Errors that can occur during encryption/decryption operations
#[derive(Debug, Error)]
pub enum CryptoError {
    /// Failed to generate random bytes
    #[error("Failed to generate random bytes")]
    RandomGenerationError,

    /// Invalid hex encoding
    #[error("Invalid hex encoding: {0}")]
    HexEncodingError(String),

    /// AES-GCM encryption error
    #[error("AES-GCM encryption error: {0}")]
    AesGcmError(String),

    /// Generic error with message
    #[error("{0}")]
    GenericError(String),
}

impl From<AesGcmError> for CryptoError {
    fn from(err: AesGcmError) -> Self {
        CryptoError::AesGcmError(err.to_string())
    }
}

/// Generates secure random encryption parameters (key and nonce)
///
/// This function creates a new set of encryption parameters consisting of
/// a 32-byte key for AES-256 encryption and a 16-byte nonce for initialization.
/// The key and nonce are generated using a cryptographically secure RNG.
///
/// # Returns
///
/// An EncryptionParams struct containing the generated key and nonce.
pub fn generate_encryption_params() -> Result<EncryptionParams, CryptoError> {
    let mut rng = rand::thread_rng();

    // Generate 32 byte key (for AES-256)
    let key = rng.gen::<[u8; 32]>();
    // Generate 16 byte nonce (to match 0xChat)
    let nonce = rng.gen::<[u8; 16]>();

    Ok(EncryptionParams {
        key: hex::encode(key),
        nonce: hex::encode(nonce),
    })
}

/// Encrypts data using AES-256-GCM with a 16-byte nonce
///
/// This function encrypts the provided data using the AES-256-GCM algorithm
/// with the given encryption parameters. The data is encrypted in place,
/// and the authentication tag is appended to the result.
///
/// # Arguments
///
/// * `data` - The data to encrypt
/// * `params` - The encryption parameters containing the key and nonce
///
/// # Returns
///
/// A Result containing the encrypted data with the authentication tag appended,
/// or a CryptoError if encryption fails.
pub fn encrypt_data(data: &[u8], params: &EncryptionParams) -> Result<Vec<u8>, CryptoError> {
    debug!("Encrypting data with key: {}", params.key);

    // Decode key and nonce from hex
    let key_bytes = hex::decode(&params.key)
        .map_err(|_| CryptoError::HexEncodingError("Invalid key".into()))?;
    let nonce_bytes = hex::decode(&params.nonce)
        .map_err(|_| CryptoError::HexEncodingError("Invalid nonce".into()))?;

    // Initialize AES-GCM cipher
    let cipher = AesGcm::<Aes256, U16>::new(GenericArray::from_slice(&key_bytes));

    // Prepare nonce
    let nonce = GenericArray::from_slice(&nonce_bytes);

    // Create output buffer
    let mut buffer = data.to_vec();

    // Encrypt in place and get authentication tag
    let tag = cipher
        .encrypt_in_place_detached(nonce, &[], &mut buffer)
        .map_err(|e| CryptoError::AesGcmError(e.to_string()))?;

    // Append the authentication tag to the encrypted data
    buffer.extend_from_slice(tag.as_slice());

    debug!("Data encrypted successfully");
    Ok(buffer)
}
