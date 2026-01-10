use mdk_core::GroupId;
use ::url::Url;
use log::{debug, error};
use nostr_sdk::prelude::*;
use std::result::Result;
use thiserror::Error;

// Re-export the Nostr client type for downstream crates
pub use nostr_sdk::prelude::Client as NostrClient;

// Clean, namespaced re-exports of commonly used Nostr SDK items so that downstream systems
// can depend only on vector_sdk.
pub mod nostr {
    pub use nostr_sdk::prelude::{
        Client, Keys, PublicKey, SecretKey, Kind, Filter, Timestamp, Event, Metadata,
        EventBuilder, Tag, TagKind, ToBech32, FromBech32,
    };
    pub use nostr_sdk::RelayPoolNotification;
    pub use nostr_sdk::nips::nip59::UnwrappedGift;
    pub use nostr_sdk::event::id;
}

pub mod mls;
pub mod blossom;

pub mod client;
pub mod crypto;
pub mod metadata;
pub mod subscription;
pub mod upload;

use crate::client::build_client;
use crate::mls::MlsGroup;

use once_cell::sync::OnceCell;
use sha2::{Digest, Sha256};
use magical_rs::magical::bytes_read::with_bytes_read;
use magical_rs::magical::magic::FileKind;

/// Comprehensive error type for the Vector SDK
#[derive(Debug, Error)]
pub enum VectorBotError {
    #[error("MLS error: {0}")]
    Mls(#[from] mls::MlsError),

    #[error("Crypto error: {0}")]
    Crypto(#[from] crate::crypto::CryptoError),

    #[error("Upload error: {0}")]
    Upload(#[from] crate::upload::UploadError),

    #[error("URL parsing error: {0}")]
    UrlParse(#[from] url::ParseError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Nostr SDK error: {0}")]
    Nostr(String),

    #[error("Serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Metadata error: {0}")]
    Metadata(#[from] crate::metadata::MetadataError),

    #[error("Subscription error: {0}")]
    Subscription(#[from] crate::subscription::SubscriptionError),
}

impl From<String> for VectorBotError {
    fn from(err: String) -> Self {
        VectorBotError::Nostr(err)
    }
}

impl From<&str> for VectorBotError {
    fn from(err: &str) -> Self {
        VectorBotError::Nostr(err.to_string())
    }
}

/// # Blossom Media Servers
///
/// A list of Blossom servers for file uploads with automatic failover.
/// The system will try each server in order until one succeeds.
static BLOSSOM_SERVERS: OnceCell<std::sync::Mutex<Vec<String>>> = OnceCell::new();

/// Initialize default Blossom servers
fn init_blossom_servers() -> Vec<String> {
    vec![
        "https://blossom.primal.net".to_string(),
    ]
}

/// Get the list of Blossom servers (internal function)
pub(crate) fn get_blossom_servers() -> Vec<String> {
    BLOSSOM_SERVERS
        .get_or_init(|| std::sync::Mutex::new(init_blossom_servers()))
        .lock()
        .unwrap()
        .clone()
}

/// A vector bot that can send and receive private messages.
///
/// This struct represents a vector bot with configured metadata and client.
/// It provides methods to send private messages and handle notifications.
#[derive(Clone)]
#[allow(dead_code)]
pub struct VectorBot {
    /// The keys used to sign messages.
    keys: Keys,

    device_mdk:mls::MlsGroup,

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
        name: impl Into<String>,
        display_name: impl Into<String>,
        about: impl Into<String>,
        picture: impl AsRef<str>,
        banner: impl AsRef<str>,
        nip05: impl Into<String>,
        lud16: impl Into<String>,
    ) -> Self {
        // Convert Into<String> to String so we can pass owned values to the helper.
        let name = name.into();
        let display_name = display_name.into();
        let about = about.into();
        let nip05 = nip05.into();
        let lud16 = lud16.into();

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
        picture: impl AsRef<str>,
        banner: impl AsRef<str>,
        nip05: String,
        lud16: String,
    ) -> Self {
        // MLS
        // Create the mdk instance
        let device_mdk = match MlsGroup::new_persistent() {
            Ok(device_mdk) => device_mdk,
            Err(e) => {
                error!("Error creating MlsGroup: {}", e);
                panic!("Failed to initialize MLS service: {}", e);
            }
        };

        let picture_url = match Url::parse(picture.as_ref()) {
            Ok(url) => url,
            Err(e) => {
                error!("Invalid picture URL: {}", e);
                panic!("Invalid picture URL: {}", e);
            }
        };

        let banner_url = match Url::parse(banner.as_ref()) {
            Ok(url) => url,
            Err(e) => {
                error!("Invalid banner URL: {}", e);
                panic!("Invalid banner URL: {}", e);
            }
        };

        let client = build_client(
            keys.clone(),
            device_mdk.clone(),
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
            device_mdk,
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

    /// Takes a welcome event and checks out the group information.
    ///
    /// This function processes a welcome event to retrieve group information
    /// and determine if it's a group the bot wants to join.
    ///
    /// # Arguments
    ///
    /// * `welcome_event` - The unsigned welcome event to process.
    ///
    /// # Returns
    ///
    /// A Result containing the group information or an error message.
    pub async fn checkout_group(&self, welcome_event: UnsignedEvent) -> Result<mdk_storage_traits::groups::types::Group, VectorBotError> {
        let engine = self.device_mdk.engine()?;

        let wrapper_event_id = welcome_event.id
            .ok_or(VectorBotError::InvalidInput("Event Id not set".to_string()))?;

        let process_welcome_result = engine.process_welcome(&wrapper_event_id, &welcome_event);

        debug!("Process Welcome Result: {:#?}", process_welcome_result);

        match process_welcome_result {
            Ok(welcome) => {
                engine.get_group(&welcome.mls_group_id)
                    .map_err(|e| VectorBotError::Storage(format!("Error accessing storage: {}", e)))?
                    .ok_or_else(|| VectorBotError::InvalidInput("No group with that id".to_string()))
            },
            Err(e) => {
                error!("Welcome didn't process correctly and couldn't be handled: {}", e);
                Err(VectorBotError::Mls(mls::MlsError::NostrMlsError(format!("Welcome processing failed: {}", e))))
            }
        }
    }

    /// Processes a group message event.
    ///
    /// This function processes a group message to extract the application message.
    ///
    /// # Arguments
    ///
    /// * `event` - The event to process.
    ///
    /// # Returns
    ///
    /// A Result containing the message or an error message.
    pub async fn process_group_message(&self, event: &Event) -> Result<mdk_core::prelude::message_types::Message, VectorBotError> {
        debug!("Processing group message");
        let engine = self.device_mdk.engine()?;

        match engine.process_message(event) {
            Ok(mdk_core::prelude::MessageProcessingResult::ApplicationMessage(msg)) => {
                Ok(msg)
            },
            Ok(_) => {
                error!("Unsupported message type");
                Err(VectorBotError::InvalidInput("Unsupported message type".to_string()))
            },
            Err(e) => {
                error!("Failed to process message: {}", e);
                Err(VectorBotError::Mls(mls::MlsError::NostrMlsError(format!("Message processing failed: {}", e))))
            }
        }
    }

    /// Joins a group by its group ID.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The ID of the group to join.
    ///
    /// # Returns
    ///
    /// A Result containing the Group or an error message.
    pub async fn join_group(&self, group_id: GroupId) -> Result<Group, VectorBotError> {
        let engine = self.device_mdk.engine()?;

        let welcome_result = engine.get_pending_welcomes()
            .map_err(|e| VectorBotError::Storage(format!("Error getting pending welcomes: {}", e)))?;

        let welcome = welcome_result.into_iter()
            .find(|wi| wi.mls_group_id == group_id)
            .ok_or_else(|| VectorBotError::InvalidInput("No welcomes available".to_string()))?;

        debug!("Found welcome: {:#?}", welcome);

        if let Err(e) = engine.accept_welcome(&welcome) {
            error!("Failed to accept welcome: {:#?}", e);
            return Err(VectorBotError::Mls(mls::MlsError::NostrMlsError(format!("Failed to accept welcome: {}", e))));
        }

        self.get_group(welcome.mls_group_id.clone()).await
    }

    /// Quickly joins a group using a welcome event.
    ///
    /// # Arguments
    ///
    /// * `welcome_event` - The welcome event to process.
    ///
    /// # Returns
    ///
    /// A Result containing the Group or an error message.
    pub async fn quick_join_group(&self, welcome_event: UnsignedEvent) -> Result<Group, VectorBotError> {
        let engine = self.device_mdk.engine()?;

        let wrapper_event_id = welcome_event.id
            .ok_or(VectorBotError::InvalidInput("Event Id not set".to_string()))?;

        engine.process_welcome(&wrapper_event_id, &welcome_event)
            .map_err(|e| VectorBotError::Mls(mls::MlsError::NostrMlsError(format!("Failed to process welcome: {}", e))))?;

        let welcomes = engine.get_pending_welcomes()
            .map_err(|e| VectorBotError::Storage(format!("Error getting pending welcomes: {}", e)))?;

        let welcome = welcomes.first()
            .ok_or_else(|| VectorBotError::InvalidInput("No welcomes available".to_string()))?;

        debug!("Found welcome: {:#?}", welcome);

        if let Err(e) = engine.accept_welcome(welcome) {
            error!("Failed to accept welcome: {:#?}", e);
            return Err(VectorBotError::Mls(mls::MlsError::NostrMlsError(format!("Failed to accept welcome: {}", e))));
        }

        self.get_group(welcome.mls_group_id.clone()).await
    }

    /// Gets a group by its ID.
    ///
    /// # Arguments
    ///
    /// * `group_id` - The ID of the group to retrieve.
    ///
    /// # Returns
    ///
    /// A Result containing the Group or an error message.
    pub async fn get_group(&self, group_id: GroupId) -> Result<Group, VectorBotError> {
        let engine = self.device_mdk.engine()?;

        let group_info = engine.get_group(&group_id)
            .map_err(|e| VectorBotError::Storage(format!("Error accessing storage: {}", e)))?
            .ok_or_else(|| VectorBotError::InvalidInput("No group with that id".to_string()))?;

        Ok(Group::new(group_info, self).await)
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

pub struct Group {
    group : mdk_core::prelude::group_types::Group,
    base_bot: VectorBot,
}
impl Group {
    // This will be all of the group functions and features
    pub async fn new(mdk_group: mdk_core::prelude::group_types::Group, bot: &VectorBot) -> Self {
        Self {
            group: mdk_group,
            base_bot: bot.clone(),
        }
    }

    /// Gets a message by its ID.
    ///
    /// # Arguments
    ///
    /// * `message_id` - The ID of the message to retrieve.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure.
    pub async fn get_message(&self, message_id: &EventId) -> Result<(), VectorBotError> {
        let engine = self.base_bot.device_mdk.engine()?;

        engine.get_message(message_id)
            .map_err(|e| VectorBotError::Storage(format!("Error finding the message: {}", e)))?;
        Ok(())
    }

    /// Checks all messages in the group.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure.
    pub async fn check_group_messages(&self) -> Result<(), VectorBotError> {
        let engine = self.base_bot.device_mdk.engine()?;

        let messages = engine.get_messages(&self.group.mls_group_id)
            .map_err(|e| VectorBotError::Storage(format!("Error getting messages: {}", e)))?;
        debug!("Found {} messages in group", messages.len());
        Ok(())
    }

    /// Sends a message to the group.
    ///
    /// # Arguments
    ///
    /// * `message` - The message content to send.
    ///
    /// # Returns
    ///
    /// A Result indicating success or failure.
    pub async fn send_group_message(&self, message: &str) -> Result<(), VectorBotError> {
        debug!("Sending a message to the group: {:?}", &self.group.mls_group_id);

        // Build a minimal inner rumor carrying the plaintext payload.
        let rumor_builder = EventBuilder::new(Kind::PrivateDirectMessage, message);
        let rumor = rumor_builder.build(self.base_bot.keys.public_key());

        let engine = self.base_bot.device_mdk.engine()?;

        let group_message_creation = engine.create_message(&self.group.mls_group_id, rumor.clone())
            .map_err(|e| VectorBotError::Mls(mls::MlsError::NostrMlsError(format!("Error creating the group message event: {}", e))))?;

        debug!("Successfully created the message");

        self.base_bot.client.send_event(&group_message_creation).await
            .map_err(|e| VectorBotError::Nostr(format!("Error sending event: {:?}", e)))?;

        Ok(())
    }

    /// Sends a typing indicator to the group.
    ///
    /// This function sends a typing indicator event to all members of the group.
    ///
    /// # Returns
    ///
    /// `true` if the typing indicator was sent successfully, `false` otherwise.
    pub async fn send_group_typing_indicator(&self) -> bool {
        debug!("Sending typing indicator to group: {:?}", &self.group.mls_group_id);

        // Build a typing indicator event
        let content = String::from("typing");
        let expiration = Timestamp::from_secs(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                + 30,
        );

        // Add millisecond precision tag
        let final_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let milliseconds = final_time.as_millis() % 1000;

        // Build the typing indicator rumor
        let rumor = EventBuilder::new(Kind::ApplicationSpecificData, content)
            .tag(Tag::custom(TagKind::d(), vec!["vector"]))
            .tag(Tag::custom(TagKind::custom("ms"), [milliseconds.to_string()]))
            .tag(Tag::expiration(expiration));

        let built_rumor = rumor.build(self.base_bot.keys.public_key());

        // Wrap the rumor in an MLS message for the group
        let engine = match self.base_bot.device_mdk.engine() {
            Ok(e) => e,
            Err(e) => {
                error!("Failed to get MLS engine: {}", e);
                return false;
            }
        };

        let group_message_creation = match engine.create_message(&self.group.mls_group_id, built_rumor.clone()) {
            Ok(msg) => msg,
            Err(e) => {
                error!("Error creating the group typing indicator event: {}", e);
                return false;
            }
        };

        debug!("Successfully created the typing indicator message");

        match self.base_bot.client.send_event(&group_message_creation).await {
            Ok(_) => true,
            Err(e) => {
                error!("Error sending typing indicator event: {:?}", e);
                false
            }
        }
    }

    pub async fn send_group_attachment(&self, file: Option<AttachmentFile>) -> Result<(), VectorBotError> {
        let servers = crate::get_blossom_servers();
        let attached_file = file.ok_or_else(|| VectorBotError::InvalidInput("No file provided for sending".to_string()))?;

        // Calculate the file hash first (before encryption)
        let file_hash = calculate_file_hash(&attached_file.bytes);

        // Format a Mime Type from the file extension
        let mime_type = get_mime_type(&attached_file.extension);

        // Generate encryption parameters and encrypt the file
        let params = crypto::generate_encryption_params()?;

        let enc_file = crypto::encrypt_data(attached_file.bytes.as_slice(), &params)?;
        let file_size = enc_file.len();

        // Create a progress callback for file uploads
        let progress_callback: crate::blossom::ProgressCallback = std::sync::Arc::new(move |percentage, _bytes| {
                if let Some(pct) = percentage {
                    debug!("Upload progress: {}%", pct);
                }
                Ok(())
            });

        let url = crate::blossom::upload_blob_with_progress_and_failover(self.base_bot.keys.clone(), servers, enc_file, Some(mime_type.as_str()), progress_callback, Some(3), Some(std::time::Duration::from_secs(2))).await?;

        let url_parsed = Url::parse(&url)?;

        // We will just build a custom rumor for now to test
        let final_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap();
        let milliseconds = final_time.as_millis() % 1000;

        // Create the attachment rumor
        let mut attachment_rumor = EventBuilder::new(Kind::from_u16(15), url_parsed.to_string())
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
        if let Some(ref img_meta) = attached_file.img_meta {
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

        let built_rumor = attachment_rumor.build(self.base_bot.keys.public_key());

        debug!("Sending attachment rumor: {:?}", built_rumor);

        let engine = self.base_bot.device_mdk.engine()?;

        let group_message_creation = engine.create_message(&self.group.mls_group_id, built_rumor.clone())
            .map_err(|e| VectorBotError::Mls(mls::MlsError::NostrMlsError(format!("Error creating the group message event: {}", e))))?;

        debug!("Successfully created the message");

        self.base_bot.client.send_event(&group_message_creation).await
            .map_err(|e| VectorBotError::Nostr(format!("Error sending event: {:?}", e)))?;

        Ok(())
    }

    /// Sends a reaction to a group message.
    ///
    /// This function sends a reaction to a specific message within a group.
    ///
    /// # Arguments
    ///
    /// * `reference_id` - The ID of the message to react to.
    /// * `emoji` - The emoji to use for the reaction.
    ///
    /// # Returns
    ///
    /// `true` if the reaction was sent successfully, `false` otherwise.
    pub async fn send_group_reaction(&self, reference_id: String, emoji: String) -> bool {
        debug!("Sending a reaction event to group: {:?}", &self.group.mls_group_id);

        if let Err(err) = send_group_nip25(
            &self.base_bot,
            &self.group.mls_group_id,
            reference_id,
            emoji,
        )
        .await
        {
            error!("Failed to send group reaction: {}", err);
            return false;
        }
        true
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

    pub async fn send_reaction(&self, reference_id: String, emoji: String) -> bool {
        debug!("Sending a reaction event to: {:?}", self.recipient);

        // We need the reference_event and the emoji, we can create the rest here

        // Create and send the kind30078 with our typing tag
        if let Err(err) = send_nip25(
            &self.base_bot,
            &self.recipient,
            reference_id,
            Kind::PrivateDirectMessage,
            emoji,
        )
        .await
        {
            error!("Failed to send attachment rumor: {}", err);
            return false;
        }
        true
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
        let servers = crate::get_blossom_servers();
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

        // Create a progress callback for file uploads
        let progress_callback: crate::blossom::ProgressCallback = std::sync::Arc::new(move |percentage, _bytes| {
                if let Some(pct) = percentage {
                    debug!("Upload progress: {}%", pct);
                }
                Ok(())
            });

        match crate::blossom::upload_blob_with_progress_and_failover(
            self.base_bot.keys.clone(),
            servers,
            enc_file,
            Some(mime_type.as_str()),
            progress_callback,
            Some(3),
            Some(std::time::Duration::from_secs(2))
        ).await {
            Ok(url) => {
                match Url::parse(&url) {
                    Ok(url_parsed) => {
                        if let Err(e) = send_attachment_rumor(
                            &self.base_bot,
                            &self.recipient,
                            &url_parsed,
                            &attached_file,
                            &params,
                            &file_hash,
                            file_size,
                            &mime_type,
                        ).await {
                            error!("Failed to send attachment rumor: {}", e);
                            return false;
                        }
                        debug!("Upload successful");
                        true
                    }
                    Err(e1) => {
                        error!("Failed to parse URL: {}", e1);
                        false
                    }
                }
            },
            Err(e) => {
                error!("[Blossom Error] Upload failed: {}", e);
                false
            }
        }
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
fn get_mime_type(extension: &str) -> String {
    // Prefer mime_guess to derive a correct MIME from the extension.
    // Fallback to application/octet-stream if unknown.
    let mime = mime_guess::from_ext(extension).first_or_octet_stream();
    mime.essence_str().to_string()
}

/**
 Infer a likely file extension using magical_rs only.
 Returns a common extension string (e.g. "png", "jpg") or None when unknown.
*/
fn infer_extension_from_bytes(bytes: &[u8]) -> Option<&'static str> {
    // Use magical_rs recommended header length
    let max = with_bytes_read();
    let header = if bytes.len() > max { &bytes[..max] } else { bytes };
    if let Some(kind) = FileKind::match_types(header) {
        let name = format!("{:?}", kind).to_lowercase();

        // Map common identifiers to standard extensions
        if name.contains("png") { return Some("png"); }
        if name.contains("jpeg") || name.contains("jpg") { return Some("jpg"); }
        if name.contains("gif") { return Some("gif"); }
        if name.contains("webp") { return Some("webp"); }
        if name.contains("bmp") { return Some("bmp"); }
        if name.contains("tiff") || name.contains("tif") { return Some("tiff"); }
        if name.contains("heic") || name.contains("heif") { return Some("heic"); }

        if name.contains("wav") || name.contains("wave") { return Some("wav"); }
        if name.contains("ogg") { return Some("ogg"); }
        if name.contains("flac") { return Some("flac"); }
        if name.contains("mp3") { return Some("mp3"); }
        if name.contains("m4a") { return Some("m4a"); }

        if name.contains("quicktime") || name.contains("mov") { return Some("mov"); }
        if name.contains("mp4") { return Some("mp4"); }
        if name.contains("webm") { return Some("webm"); }
        if name.contains("matroska") || name.contains("mkv") { return Some("mkv"); }
        if name.contains("avi") { return Some("avi"); }

        if name.contains("pdf") { return Some("pdf"); }
        if name.contains("zip") { return Some("zip"); }
        if name.contains("iso") { return Some("iso"); }
        if name.contains("7z") { return Some("7z"); }
        if name.contains("tar") { return Some("tar"); }
        if name.contains("gzip") || name.contains("gz") { return Some("gz"); }
        if name.contains("bz2") { return Some("bz2"); }
        if name.contains("xz") { return Some("xz"); }
    }

    None
}

/// Sends a reaction to a group message
///
/// # Arguments
///
/// * `bot` - A reference to the VectorBot.
/// * `group_id` - The group ID to send the reaction to.
/// * `reference_id` - The ID of the message to react to.
/// * `emoji` - The emoji to use for the reaction.
///
/// # Returns
///
/// A Result indicating success or failure.
async fn send_group_nip25(bot: &VectorBot, group_id: &GroupId, reference_id: String, emoji: String) -> Result<(), VectorBotError> {
    let reference_event = EventId::from_hex(reference_id.as_str())
        .map_err(|e| VectorBotError::InvalidInput(format!("Invalid reference ID: {}", e)))?;

    // Create the reaction event
    let rumor = EventBuilder::reaction_extended(
        reference_event,
        bot.keys.public_key(),
        None, // No specific message type for groups
        &emoji,
    );

    let built_rumor = rumor.build(bot.keys.public_key());

    // Wrap the rumor in an MLS message for the group
    let engine = bot.device_mdk.engine()?;

    let group_message_creation = engine.create_message(group_id, built_rumor.clone())
        .map_err(|e| VectorBotError::Mls(mls::MlsError::NostrMlsError(format!("Error creating the group reaction event: {}", e))))?;

    debug!("Successfully created the group reaction message");

    bot.client.send_event(&group_message_creation).await
        .map_err(|e| VectorBotError::Nostr(format!("Error sending group reaction event: {:?}", e)))?;

    Ok(())
}

async fn send_nip25(bot: &VectorBot, recipient: &PublicKey, reference_id: String, message_type: Kind, emoji: String) -> Result<(), VectorBotError> {
    let reference_event = EventId::from_hex(reference_id.as_str())
        .map_err(|e| VectorBotError::InvalidInput(format!("Invalid reference ID: {}", e)))?;

    let rumor = EventBuilder::reaction_extended(
        reference_event,
        *recipient,
        Some(message_type),
        &emoji,
    );

    let built_rumor = rumor.build(bot.keys.public_key());

    match bot
        .client
        .gift_wrap(recipient, built_rumor.clone(), [],)
        .await
    {
        Ok(output) => {
            if output.success.is_empty() && !output.failed.is_empty() {
                error!("Failed to send attachment rumor: {:?}", output);
                return Err(VectorBotError::Nostr("Failed to send attachment rumor".to_string()));
            }
            Ok(())
        }
        Err(e) => {
            error!("Error sending attachment rumor: {:?}", e);
            Err(VectorBotError::Nostr(format!("Error sending attachment rumor: {:?}", e)))
        }
    }
}

async fn send_kind30078(bot: &VectorBot, recipient: &PublicKey, content: String, expiration: Timestamp) -> Result<(), VectorBotError> {
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
                return Err(VectorBotError::Nostr("Failed to send attachment rumor".to_string()));
            }
            Ok(())
        }
        Err(e) => {
            error!("Error sending attachment rumor: {:?}", e);
            Err(VectorBotError::Nostr(format!("Error sending attachment rumor: {:?}", e)))
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
) -> Result<(), VectorBotError> {
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
                return Err(VectorBotError::Nostr("Failed to send attachment rumor".to_string()));
            }
            Ok(())
        }
        Err(e) => {
            error!("Error sending attachment rumor: {:?}", e);
            Err(VectorBotError::Nostr(format!("Error sending attachment rumor: {:?}", e)))
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

/// Load a file from disk into an AttachmentFile, using mime_guess to infer a sensible extension
/// when the path has none or is unknown.
pub fn load_file<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<AttachmentFile> {
    let path_ref = path.as_ref();

    // Read bytes from disk
    let bytes = std::fs::read(path_ref)?;

    // Prefer filesystem extension; if absent/invalid, derive from MIME guess
    let extension = path_ref
        .extension()
        .and_then(|s| s.to_str())
        .map(|s| s.to_string())
        .or_else(|| {
            let mime = mime_guess::from_path(path_ref).first_or_octet_stream();
            mime_guess::get_mime_extensions(&mime)
                .and_then(|arr| arr.first().map(|e| (*e).to_string()))
        })
        .unwrap_or_else(|| "bin".to_string());

    Ok(AttachmentFile {
        bytes,
        img_meta: None,
        extension,
    })
}

impl AttachmentFile {
    /// Create an AttachmentFile directly from a path on disk.
    /// Equivalent to calling [`rust.load_file()`](src/lib.rs:682).
    pub fn from_path<P: AsRef<std::path::Path>>(path: P) -> std::io::Result<Self> {
        load_file(path)
    }

    /// Create an AttachmentFile from in-memory bytes.
    /// Attempts to infer a sensible file extension via byte sniffing, falling back to "bin".
    pub fn from_bytes<B: Into<Vec<u8>>>(bytes: B) -> Self {
        let bytes_vec = bytes.into();
        let ext = infer_extension_from_bytes(&bytes_vec)
            .unwrap_or("bin")
            .to_string();
        Self {
            bytes: bytes_vec,
            img_meta: None,
            extension: ext,
        }
    }
}
