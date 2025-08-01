use crate::rand;
use crate::rand::Rng;
use aes::Aes256;
use aes_gcm::{AesGcm, AeadInPlace, KeyInit};
use generic_array::{GenericArray, typenum::U16};

/// Represents encryption parameters
#[derive(Debug)]
pub struct EncryptionParams {
    /// The encryption key (hex string)
    pub key: String,
    /// The initialization vector (nonce) (hex string)
    pub nonce: String,
}

/// Generates random encryption parameters (key and nonce)
///
/// This function creates a new set of encryption parameters consisting of
/// a 32-byte key for AES-256 encryption and a 16-byte nonce for initialization.
///
/// # Returns
///
/// An EncryptionParams struct containing the generated key and nonce.
pub fn generate_encryption_params() -> EncryptionParams {
    let mut rng = rand::thread_rng();

    // Generate 32 byte key (for AES-256)
    let key: [u8; 32] = rng.gen();
    // Generate 16 byte nonce (to match 0xChat)
    let nonce: [u8; 16] = rng.gen();

    EncryptionParams {
        key: hex::encode(key),
        nonce: hex::encode(nonce),
    }
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
/// or a String containing an error message if encryption fails.
pub fn encrypt_data(data: &[u8], params: &EncryptionParams) -> Result<Vec<u8>, String> {
    // Decode key and nonce from hex
    let key_bytes = hex::decode(&params.key).unwrap();
    let nonce_bytes = hex::decode(&params.nonce).unwrap();

    // Initialize AES-GCM cipher
    let cipher = AesGcm::<Aes256, U16>::new(
        GenericArray::from_slice(&key_bytes)
    );

    // Prepare nonce
    let nonce = GenericArray::from_slice(&nonce_bytes);

    // Create output buffer
    let mut buffer = data.to_vec();

    // Encrypt in place and get authentication tag
    let tag = cipher
        .encrypt_in_place_detached(nonce, &[], &mut buffer)
        .map_err(|_| String::from("Failed to Encrypt Data"))?;

    // Append the authentication tag to the encrypted data
    buffer.extend_from_slice(tag.as_slice());

    Ok(buffer)
}