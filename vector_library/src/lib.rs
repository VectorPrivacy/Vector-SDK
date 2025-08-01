extern crate nostr_sdk;
use nostr_sdk::prelude::*;
use url::Url;

pub mod client;
pub mod metadata;
pub mod subscription;
pub mod crypto;
pub mod upload;

use once_cell::sync::OnceCell;
use crate::client::build_client;
use sha2::{Sha256, Digest};

static TRUSTED_PRIVATE_NIP96: &str = "https://medea-1-swiss.vectorapp.io";
static PRIVATE_NIP96_CONFIG: OnceCell<ServerConfig> = OnceCell::new();

/// A vector bot that can send and receive private messages.
///
/// This struct represents a vector bot with configured metadata and client.
/// It provides methods to send private messages and handle notifications.
#[derive(Clone)]
pub struct VectorBot {
    /// The keys used to sign messages.
    #[allow(dead_code)]
    keys: Keys,

    /// The name of the user.
    #[allow(dead_code)]
    name: String,

    /// The display name of the user.
    #[allow(dead_code)]
    display_name: String,

    /// A brief description about the user.
    #[allow(dead_code)]
    about: String,

    /// The URL of the user's profile picture.
    #[allow(dead_code)]
    picture: Url,

    /// The URL of the user's banner.
    #[allow(dead_code)]
    banner: Url,

    /// The NIP05 identifier.
    #[allow(dead_code)]
    nip05: String,

    /// The LUD16 payment pointer.
    #[allow(dead_code)]
    lud16: String,

    /// The vector client.
    pub client: Client,
}

pub struct Channel {
    recipient: PublicKey,
    base_bot: VectorBot,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct ImageMetadata {
    /// The Blurhash preview
    pub blurhash: String,
    /// Image pixel width
    pub width: u32,
    /// Image pixel height
    pub height: u32,
}

/// A simple pre-upload format to associate a byte stream with a file extension
#[derive(serde::Serialize, serde::Deserialize, Clone, Debug)]
pub struct AttachmentFile {
    pub bytes: Vec<u8>,
    // /// Image metadata (for images only)
    pub img_meta: Option<ImageMetadata>,
    pub extension: String,
}

/// Calculate SHA-256 hash of file data
pub fn calculate_file_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
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
        let metadata_name = "username".to_string();
        let metadata_display_name = "My Username".to_string();
        let metadata_about = "Description".to_string();
        let metadata_picture = Url::parse("https://example.com/avatar.png").expect("Invalid URL");
        let metadata_banner = Url::parse("https://example.com/banner.png").expect("Invalid URL");
        let metadata_nip05 = "username@example.com".to_string();
        let metadata_lud16 = "pay@yukikishimoto.com".to_string();

        let client = build_client(
            keys.clone(),
            metadata_name.clone(),
            metadata_display_name.clone(),
            metadata_about.clone(),
            metadata_picture.clone(),
            metadata_banner.clone(),
            metadata_nip05.clone(),
            metadata_lud16.clone(),
        ).await;

        Self {
            keys,
            name: metadata_name,
            display_name: metadata_display_name,
            about: metadata_about,
            picture: metadata_picture,
            banner: metadata_banner,
            nip05: metadata_nip05,
            lud16: metadata_lud16,
            client,
        }
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
        let picture_url = Url::parse(picture).expect("Invalid URL");
        let banner_url = Url::parse(banner).expect("Invalid URL");

        let client = build_client(
            keys.clone(),
            name.clone(),
            display_name.clone(),
            about.clone(),
            picture_url.clone(),
            banner_url.clone(),
            nip05.clone(),
            lud16.clone(),
        ).await;

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
        let chat = Channel::new(chat_npub, self).await;
        chat
    }
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
        println!("pubkey: {:#?}", self.recipient);
        match self.base_bot.client.send_private_msg(self.recipient, message, []).await {
            Ok(output) => {
                println!("{:#?}",output);
                true
            },
            Err(_) => false,
        }
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
        let attached_file = file.unwrap();

        // Calculate the file hash first (before encryption)
        let file_hash = calculate_file_hash(&attached_file.bytes);

        let mut final_output = None;

        // Format a Mime Type from the file extension
        let mime_type = match attached_file.extension.as_str() {
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
        };

        let params = crypto::generate_encryption_params();
        let enc_file = crypto::encrypt_data(attached_file.bytes.as_slice(), &params).unwrap();
        let file_size = enc_file.len();

        use nostr_sdk::nips::nip96::get_server_config;

        let _ = match get_server_config(Url::parse(TRUSTED_PRIVATE_NIP96).unwrap(), None).await {
            Ok(conf) => PRIVATE_NIP96_CONFIG.set(conf),
            Err(_) => return false
        };

        // Create a progress callback for file uploads
        let progress_callback: crate::upload::ProgressCallback = Box::new(move |percentage, _| {
                if let Some(pct) = percentage {
                    println!("{:#?}", percentage)
                }
            Ok(())
        });

        let conf = PRIVATE_NIP96_CONFIG.wait();
        let final_attachment_rumor = {
            // Upload the file with both a Progress Emitter and multiple re-try attempts in case of connection instability
            match crate::upload::upload_data_with_progress(&self.base_bot.keys, &conf, enc_file, Some(mime_type), None, progress_callback, Some(3), Some(std::time::Duration::from_secs(2))).await {
                Ok(url) => {

                    // Create the attachment rumor
                    let mut attachment_rumor = EventBuilder::new(Kind::from_u16(15), url.to_string())

                    // Append decryption keys and file metadata
                        .tag(Tag::public_key(self.recipient))
                        .tag(Tag::custom(TagKind::custom("file-type"), [mime_type]))
                        .tag(Tag::custom(TagKind::custom("size"), [file_size.to_string()]))
                        .tag(Tag::custom(TagKind::custom("encryption-algorithm"), ["aes-gcm"]))
                        .tag(Tag::custom(TagKind::custom("decryption-key"), [params.key.as_str()]))
                        .tag(Tag::custom(TagKind::custom("decryption-nonce"), [params.nonce.as_str()]))
                        .tag(Tag::custom(TagKind::custom("ox"), [file_hash.clone()]));

                    // Append image metadata if available
                    if let Some(ref img_meta) = attached_file.img_meta {
                        attachment_rumor = attachment_rumor
                            .tag(Tag::custom(TagKind::custom("blurhash"), [&img_meta.blurhash]))
                            .tag(Tag::custom(TagKind::custom("dim"), [format!("{}x{}", img_meta.width, img_meta.height)]));
                    }

                    println!("Rumor: {:#?}",attachment_rumor);
                    attachment_rumor
                },
                Err(_) => {
                    return false
                }
            }

        };

        let built_rumor = final_attachment_rumor.build(self.base_bot.keys.public_key());

        println!("{:#?}", &self.recipient);
        println!("{:#?}", built_rumor.clone());
        match self.base_bot.client
            .gift_wrap(&self.recipient, built_rumor.clone(), [])
            .await
        {
            Ok(output) => {
                println!("SENT: {:#?}",output);
                // Check if at least one relay acknowledged the message
                if !output.success.is_empty() {
                    // Success! Message was acknowledged by at least one relay
                    println!("Success");
                    final_output = Some(output);
                    return true;
                } else if output.failed.is_empty() {
                    // No success but also no failures - this might be a temporary network issue
                    // Continue retrying
                    println!("No success but no failures");
                } else {
                    println!("Failed");
                    // We have failures but no successes
                    // Final attempt failed
                    return false;
                }
            }
            Err(e) => {
                // Network or other error - log and retry if we haven't exceeded attempts
                eprintln!("Failed to send message: {:?}", e);

                return false;
            }
        }

        false
    }
}