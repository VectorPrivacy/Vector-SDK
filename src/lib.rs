use ::url::Url;
use log::{debug, error};
use nostr_sdk::prelude::*;

pub mod client;
pub mod crypto;
pub mod metadata;
pub mod subscription;
pub mod upload;

use crate::client::build_client;
use once_cell::sync::OnceCell;
use sha2::{Digest, Sha256};

static TRUSTED_PRIVATE_NIP96: &str = "https://medea-1-swiss.vectorapp.io";
static PRIVATE_NIP96_CONFIG: OnceCell<ServerConfig> = OnceCell::new();

/// A vector bot that can send and receive private messages.
///
/// This struct represents a vector bot with configured metadata and client.
/// It provides methods to send private messages and handle notifications.
#[derive(Clone)]
#[allow(dead_code)]
pub struct VectorBot {
    /// The keys used to sign messages.
    keys: Keys,

    /// The name of the user.
    name: String,

    /// The display name of the user.
    display_name: String,

    /// A brief description about the user.
    about: String,

    /// The URL of the user's profile picture.
    picture: Url,

    /// The URL of the user's banner.
    banner: Url,

    /// The NIP05 identifier.
    nip05: String,

    /// The LUD16 payment pointer.
    lud16: String,

    /// The vector client.
    pub client: Client,
}

impl VectorBot {
    /// Creates a new VectorBot with default metadata.
    ///
    /// This function generates a new VectorBot with default metadata values.
    /// It's a convenient way to quickly create a bot without specifying all metadata.
    ///
    /// # Arguments
    ///
    /// * `keys` - The keys used to sign messages.
    ///
    /// # Returns
    ///
    /// A new VectorBot instance with default metadata.
    pub async fn quick(keys: Keys) -> Self {
        Self::new_with_urls(
            keys,
            "vector bot".to_string(),
            "Vector Bot".to_string(),
            "vector bot created with quick".to_string(),
            "https://example.com/avatar.png",
            "https://example.com/banner.png",
            "example@example.com".to_string(),
            "example@example.com".to_string(),
        )
        .await
    }

    /// Creates a new VectorBot with custom metadata.
    ///
    /// This function generates a new VectorBot with the provided metadata values.
    /// It allows for customization of all metadata fields.
    ///
    /// # Arguments
    ///
    /// * `keys` - The keys used to sign messages.
    /// * `name` - The name of the user.
    /// * `display_name` - The display name of the user.
    /// * `about` - A brief description about the user.
    /// * `picture` - The URL of the user's profile picture.
    /// * `banner` - The URL of the user's banner.
    /// * `nip05` - The NIP05 identifier.
    /// * `lud16` - The LUD16 payment pointer.
    ///
    /// # Returns
    ///
    /// A new VectorBot instance with the specified metadata.
    pub async fn new(
        keys: Keys,
        name: String,
        display_name: String,
        about: String,
        picture: &str,
        banner: &str,
        nip05: String,
        lud16: String,
    ) -> Self {
        Self::new_with_urls(
            keys,
            name,
            display_name,
            about,
            picture,
            banner,
            nip05,
            lud16,
        )
        .await
    }

    /// Creates a new VectorBot with the given metadata.
    ///
    /// This is a helper function that handles URL parsing and client building.
    async fn new_with_urls(
        keys: Keys,
        name: String,
        display_name: String,
        about: String,
        picture: &str,
        banner: &str,
        nip05: String,
        lud16: String,
    ) -> Self {
        let picture_url = match Url::parse(picture) {
            Ok(url) => url,
            Err(e) => {
                error!("Invalid picture URL: {}", e);
                return Self {
                    keys: keys.clone(),
                    name,
                    display_name,
                    about,
                    picture: Url::parse("https://example.com/default.png").unwrap(),
                    banner: Url::parse("https://example.com/default.png").unwrap(),
                    nip05,
                    lud16,
                    client: Client::builder().signer(keys.clone()).build(),
                };
            }
        };

        let banner_url = match Url::parse(banner) {
            Ok(url) => url,
            Err(e) => {
                error!("Invalid banner URL: {}", e);
                return Self {
                    keys: keys.clone(),
                    name,
                    display_name,
                    about,
                    picture: picture_url,
                    banner: Url::parse("https://example.com/default.png").unwrap(),
                    nip05,
                    lud16,
                    client: Client::builder().signer(keys.clone()).build(),
                };
            }
        };

        let client = build_client(
            keys.clone(),
            name.clone(),
            display_name.clone(),
            about.clone(),
            picture_url.clone(),
            banner_url.clone(),
            nip05.clone(),
            lud16.clone(),
            None,
        )
        .await;

        Self {
            keys,
            name,
            display_name,
            about,
            picture: picture_url,
            banner: banner_url,
            nip05,
            lud16,
            client,
        }
    }

    /// Gets a chat channel for a specific public key.
    ///
    /// This function creates a new Channel instance for communicating with
    /// a specific recipient.
    ///
    /// # Arguments
    ///
    /// * `chat_npub` - The public key of the recipient.
    ///
    /// # Returns
    ///
    /// A Channel instance for communicating with the specified recipient.
    pub async fn get_chat(&self, chat_npub: PublicKey) -> Channel {
        Channel::new(chat_npub, self).await
    }
}

/// Represents a communication channel with a specific recipient.
pub struct Channel {
    recipient: PublicKey,
    base_bot: VectorBot,
}

impl Channel {
    /// Creates a new Channel for communicating with a specific recipient.
    ///
    /// # Arguments
    ///
    /// * `chat_npub` - The public key of the recipient.
    /// * `bot` - A reference to the VectorBot instance.
    ///
    /// # Returns
    ///
    /// A new Channel instance.
    pub async fn new(chat_npub: PublicKey, bot: &VectorBot) -> Self {
        Self {
            recipient: chat_npub,
            base_bot: bot.clone(),
        }
    }

    /// Sends a private message to the recipient.
    ///
    /// # Arguments
    ///
    /// * `message` - The message content to send.
    ///
    /// # Returns
    ///
    /// `true` if the message was sent successfully, `false` otherwise.
    pub async fn send_private_message(&self, message: &str) -> bool {
        debug!("Sending private message to: {:?}", self.recipient);

        // Add millisecond precision tag so clients can order messages sent within the same second
        let final_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let milliseconds = final_time.as_millis() % 1000;

        match self
            .base_bot
            .client
            .send_private_msg(
                self.recipient,
                message,
                [Tag::custom(TagKind::custom("ms"), [milliseconds.to_string()])],
            )
            .await
        {
            Ok(_) => true,
            Err(e) => {
                error!("Failed to send private message: {:?}", e);
                false
            }
        }
    }


    // Sends a typing indicator
    pub async fn send_typing_indicator(&self)-> bool {
        debug!("Sending kind 30078 typing indicator to: {:?}", self.recipient);

        // We need to send "typing" & an expiration
        let content = String::from("typing");
        // For expiration lets just set max for now
        let expiration = Timestamp::from_secs(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 30,
        );

        // Create and send the kind30078 with our typing tag
        if let Err(err) = send_kind30078(
            &self.base_bot,
            &self.recipient,
            content,
            expiration,
        )
        .await
        {
            error!("Failed to send attachment rumor: {}", err);
            return false;
        }
        true
    }


    /// Sends a private file to the recipient.
    ///
    /// This function handles file encryption, uploads the file to a server,
    /// and sends a notification to the recipient with the file information.
    ///
    /// # Arguments
    ///
    /// * `file` - The file to send, wrapped in an Option.
    ///
    /// # Returns
    ///
    /// `true` if the file was sent successfully, `false` otherwise.
    pub async fn send_private_file(&self, file: Option<AttachmentFile>) -> bool {
        let attached_file = match file {
            Some(f) => f,
            None => {
                error!("No file provided for sending");
                return false;
            }
        };

        // Calculate the file hash first (before encryption)
        let file_hash = calculate_file_hash(&attached_file.bytes);

        // Format a Mime Type from the file extension
        let mime_type = get_mime_type(&attached_file.extension);

        // Generate encryption parameters and encrypt the file
        let params_result = crypto::generate_encryption_params();
        let params = match params_result {
            Ok(p) => p,
            Err(err) => {
                error!("Failed to generate encryption parameters: {}", err);
                return false;
            }
        };

        let enc_file = match crypto::encrypt_data(attached_file.bytes.as_slice(), &params) {
            Ok(data) => data,
            Err(err) => {
                error!("Failed to encrypt file: {}", err);
                return false;
            }
        };
        let file_size = enc_file.len();

        // Get server config
        let conf = match get_server_config().await {
            Ok(c) => c,
            Err(err) => {
                error!("Failed to get server config: {}", err);
                return false;
            }
        };

        // Create a progress callback for file uploads
        let progress_callback = create_progress_callback();

        // Upload the file
        let url = match upload_file(
            &self.base_bot.keys,
            &conf,
            &enc_file,
            &mime_type,
            progress_callback,
        )
        .await
        {
            Ok(u) => u,
            Err(err) => {
                error!("Failed to upload file: {}", err);
                return false;
            }
        };

        // Create and send the attachment rumor
        if let Err(err) = send_attachment_rumor(
            &self.base_bot,
            &self.recipient,
            &url,
            &attached_file,
            &params,
            &file_hash,
            file_size,
            &mime_type,
        )
        .await
        {
            error!("Failed to send attachment rumor: {}", err);
            return false;
        }

        true
    }
}

/// Derives the MIME type from a file extension.
///
/// # Arguments
///
/// * `extension` - The file extension.
///
/// # Returns
///
/// The MIME type as a string.
fn get_mime_type(extension: &str) -> &str {
    match extension.to_lowercase().as_str() {
        // Images
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "gif" => "image/gif",
        "webp" => "image/webp",
        // Audio
        "wav" => "audio/wav",
        "mp3" => "audio/mp3",
        "flac" => "audio/flac",
        "ogg" => "audio/ogg",
        "m4a" => "audio/mp4",
        "aac" => "audio/aac",
        // Videos
        "mp4" => "video/mp4",
        "webm" => "video/webm",
        "mov" => "video/quicktime",
        "avi" => "video/x-msvideo",
        "mkv" => "video/x-matroska",
        // Unknown
        _ => "application/octet-stream",
    }
}

/// Creates a progress callback for file uploads.
///
/// # Returns
///
/// A boxed progress callback function.
fn create_progress_callback() -> crate::upload::ProgressCallback {
    Box::new(move |percentage, _| {
        if let Some(pct) = percentage {
            println!("Upload progress: {}%", pct);
        }
        Ok(())
    })
}

/// Gets the server configuration for file uploads.
///
/// # Returns
///
/// A Result containing the server configuration.
async fn get_server_config() -> Result<ServerConfig, String> {
    let url = Url::parse(TRUSTED_PRIVATE_NIP96).map_err(|_| "Invalid URL")?;
    if PRIVATE_NIP96_CONFIG.get().is_some() {
        let conf = PRIVATE_NIP96_CONFIG.get().unwrap().clone();
        Ok(conf)
    }else{
        let conf = nostr_sdk::nips::nip96::get_server_config(url, None)
            .await
            .map_err(|e| e.to_string())?;
            PRIVATE_NIP96_CONFIG
                .set(conf.clone())
                .map_err(|_| "Failed to set server config")?;
        Ok(conf)
    }
}

/// Uploads a file to the server with progress tracking.
///
/// # Arguments
///
/// * `keys` - The keys for authentication.
/// * `conf` - The server configuration.
/// * `file_data` - The file data to upload.
/// * `mime_type` - The MIME type of the file.
/// * `progress_callback` - The progress callback function.
///
/// # Returns
///
/// A Result containing the URL of the uploaded file.
async fn upload_file(
    keys: &Keys,
    conf: &ServerConfig,
    file_data: &[u8],
    mime_type: &str,
    progress_callback: crate::upload::ProgressCallback,
) -> Result<Url, String> {
    let _retry_count = 3;
    let _retry_spacing = std::time::Duration::from_secs(2);

    let upload_config = upload::UploadConfig::default();
    let upload_params = upload::UploadParams::default();

    crate::upload::upload_data_with_progress(
        keys,
        conf,
        file_data.to_vec(),
        Some(mime_type),
        None,
        progress_callback,
        Some(upload_params),
        Some(upload_config),
    )
    .await
    .map_err(|e| e.to_string())
}

async fn send_kind30078(bot: &VectorBot, recipient: &PublicKey, content: String, expiration: Timestamp)-> Result<(), String> {

    // Build and broadcast the Typing Indicator
    // Add millisecond precision tag so clients can order messages sent within the same second
    let final_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let milliseconds = final_time.as_millis() % 1000;

    let rumor = EventBuilder::new(Kind::ApplicationSpecificData, content)
        .tag(Tag::public_key(*recipient))
        .tag(Tag::custom(TagKind::d(), vec!["vector"]))
        .tag(Tag::custom(TagKind::custom("ms"), [milliseconds.to_string()]))
        .tag(Tag::expiration(expiration));

    // This expiration time is for NIP-40 relays so they can purge old Typing Indicators
    let expiry_time = Timestamp::from_secs(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            + 3600,
    );

    let built_rumor = rumor.build(bot.keys.public_key());

    match bot
        .client
        .gift_wrap(recipient, built_rumor.clone(), [Tag::expiration(expiry_time)],)
        .await
    {
        Ok(output) => {
            if output.success.is_empty() && !output.failed.is_empty() {
                error!("Failed to send attachment rumor: {:?}", output);
                return Err("Failed to send attachment rumor".to_string());
            }
            Ok(())
        }
        Err(e) => {
            error!("Error sending attachment rumor: {:?}", e);
            Err(format!("Error sending attachment rumor: {:?}", e))
        }
    }

}


/// Sends an attachment rumor to the recipient.
///
/// # Arguments
///
/// * `bot` - A reference to the VectorBot.
/// * `recipient` - The recipient's public key.
/// * `url` - The URL of the uploaded file.
/// * `file` - A reference to the AttachmentFile.
/// * `params` - A reference to the encryption parameters.
/// * `file_hash` - The hash of the file.
/// * `file_size` - The size of the file.
/// * `mime_type` - The MIME type of the file.
///
/// # Returns
///
/// A Result indicating success or failure.
async fn send_attachment_rumor(
    bot: &VectorBot,
    recipient: &PublicKey,
    url: &Url,
    file: &AttachmentFile,
    params: &crypto::EncryptionParams,
    file_hash: &str,
    file_size: usize,
    mime_type: &str,
) -> Result<(), String> {
    // Add millisecond precision tag so clients can order messages sent within the same second
    let final_time = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap();
    let milliseconds = final_time.as_millis() % 1000;

    // Create the attachment rumor
    let mut attachment_rumor = EventBuilder::new(Kind::from_u16(15), url.to_string())
        .tag(Tag::public_key(*recipient))
        .tag(Tag::custom(TagKind::custom("file-type"), [mime_type]))
        .tag(Tag::custom(
            TagKind::custom("size"),
            [file_size.to_string()],
        ))
        .tag(Tag::custom(
            TagKind::custom("encryption-algorithm"),
            ["aes-gcm"],
        ))
        .tag(Tag::custom(
            TagKind::custom("decryption-key"),
            [params.key.as_str()],
        ))
        .tag(Tag::custom(
            TagKind::custom("decryption-nonce"),
            [params.nonce.as_str()],
        ))
        .tag(Tag::custom(TagKind::custom("ox"), [file_hash]))
        .tag(Tag::custom(TagKind::custom("ms"), [milliseconds.to_string()]));

    // Append image metadata if available
    if let Some(ref img_meta) = file.img_meta {
        attachment_rumor = attachment_rumor
            .tag(Tag::custom(
                TagKind::custom("blurhash"),
                [&img_meta.blurhash],
            ))
            .tag(Tag::custom(
                TagKind::custom("dim"),
                [format!("{}x{}", img_meta.width, img_meta.height)],
            ));
    }

    let built_rumor = attachment_rumor.build(bot.keys.public_key());

    debug!("Sending attachment rumor: {:?}", built_rumor);

    match bot
        .client
        .gift_wrap(recipient, built_rumor.clone(), [])
        .await
    {
        Ok(output) => {
            if output.success.is_empty() && !output.failed.is_empty() {
                error!("Failed to send attachment rumor: {:?}", output);
                return Err("Failed to send attachment rumor".to_string());
            }
            Ok(())
        }
        Err(e) => {
            error!("Error sending attachment rumor: {:?}", e);
            Err(format!("Error sending attachment rumor: {:?}", e))
        }
    }
}

/// Calculate SHA-256 hash of file data
pub fn calculate_file_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

/// Represents metadata about an image file.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct ImageMetadata {
    /// The Blurhash preview
    pub blurhash: String,
    /// Image pixel width
    pub width: u32,
    /// Image pixel height
    pub height: u32,
}

/// Represents a file attachment with metadata.
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AttachmentFile {
    /// The file bytes
    pub bytes: Vec<u8>,
    /// Image metadata (for images only)
    pub img_meta: Option<ImageMetadata>,
    /// The file extension
    pub extension: String,
}
