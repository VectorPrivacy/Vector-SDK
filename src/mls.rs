use serde::{Deserialize, Serialize};
use std::sync::Arc;
use mdk_core::prelude::*;
use mdk_sqlite_storage::MdkSqliteStorage;

#[derive(Debug)]
pub enum MlsError {
    NotInitialized,
    InvalidGroupId,
    InvalidKeyPackage,
    GroupNotFound,
    MemberNotFound,
    StorageError(String),
    NetworkError(String),
    CryptoError(String),
    NostrMlsError(String),
}

impl std::fmt::Display for MlsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MlsError::NotInitialized => write!(f, "MLS service not initialized"),
            MlsError::InvalidGroupId => write!(f, "Invalid group ID"),
            MlsError::InvalidKeyPackage => write!(f, "Invalid key package"),
            MlsError::GroupNotFound => write!(f, "Group not found"),
            MlsError::MemberNotFound => write!(f, "Member not found"),
            MlsError::StorageError(e) => write!(f, "Storage error: {}", e),
            MlsError::NetworkError(e) => write!(f, "Network error: {}", e),
            MlsError::CryptoError(e) => write!(f, "Crypto error: {}", e),
            MlsError::NostrMlsError(e) => write!(f, "Nostr MLS error: {}", e),
        }
    }
}

impl std::error::Error for MlsError {}


/// MLS group metadata stored encrypted in "mls_groups"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MlsGroupMetadata {
    // Wire identifier used on the relay (wrapper 'h' tag). UI lists this value.
    pub group_id: String,
    // Engine identifier used locally by nostr-mls for group state lookups.
    // Backwards compatible with existing data via serde default.
    #[serde(default)]
    pub engine_group_id: String,
    pub creator_pubkey: String,
    pub name: String,
    pub avatar_ref: Option<String>,
    pub created_at: u64,
    pub updated_at: u64,
    // Flag indicating if we were evicted/kicked from this group
    // When true, we skip syncing this group (unless it's a new welcome/invite)
    #[serde(default)]
    pub evicted: bool,
}


/// Event cursor tracking for a group stored in "mls_event_cursors"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventCursor {
    last_seen_event_id: String,
    last_seen_at: u64,
}

/// Message record for persisting decrypted MLS messages
/// Main MLS service facade
/// 
/// Responsibilities:
/// - Initialize and manage MLS groups using nostr-mls
/// - Handle device keypackage publishing and management
/// - Process incoming MLS events from nostr relays
/// - Manage encrypted group metadata and message storage
#[derive(Clone)]
pub struct MlsGroup {
    /// Persistent MLS engine when initialized (SQLite-backed via mdk-sqlite-storage)
    engine: Option<Arc<MDK<MdkSqliteStorage>>>,
    _initialized: bool,
}
impl MlsGroup {
    /// Create a new MLS service instance (no engine initialized)
    pub fn new() -> Self {
        Self {
            engine: None,
            _initialized: false,
        }
    }

    /// Create a new MLS service with persistent SQLite-backed storage at:
    ///   [AppData]/vector/mls/vector-mls.db
    pub fn new_persistent() -> Result<Self, MlsError> {
        // Initialize persistent storage and engine
        let storage = MdkSqliteStorage::new("mls/vector-mls.db")
            .map_err(|e| MlsError::StorageError(format!("init sqlite storage: {}", e)))?;
        let mdk = MDK::new(storage);

        Ok(Self {
            engine: Some(Arc::new(mdk)),
            _initialized: true,
        })
    }
    /// Get a clone of the persistent MLS engine (Arc)
    pub fn engine(&self) -> Result<Arc<MDK<MdkSqliteStorage>>, MlsError> {
        self.engine.clone().ok_or(MlsError::NotInitialized)
    }


    pub async fn publish_device_keypackage(&self, device_id: &str) -> Result<(), MlsError> {
        // Currently this is automatically done in the client.rs file
        let _ = device_id;
        Ok(())
    }

    /// Creates a new MLS group with the current device as the creator.
    ///
    /// # Returns
    /// Result containing the created group ID or an error
    pub async fn create_group(&self) -> Result<String, MlsError> {
        // Stub implementation - actual implementation depends on mdk-core API
        // This is a placeholder to satisfy the TODO
        Err(MlsError::NostrMlsError("create_group not yet implemented - requires mdk-core API details".to_string()))
    }

    /// Adds a member device to an existing MLS group.
    ///
    /// # Arguments
    /// * `group_id` - The ID of the group to add the member to
    /// * `member_pubkey` - The public key of the member to add
    /// * `device_id` - The device ID of the member to add
    /// * `keypackage_ref` - The reference to the member's key package
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn add_member_device(
        &self,
        group_id: &str,
        member_pubkey: &str,
        device_id: &str,
        keypackage_ref: &str,
    ) -> Result<(), MlsError> {
        // Stub implementation - actual implementation depends on mdk-core API
        // This is a placeholder to satisfy the TODO
        let _ = (group_id, member_pubkey, device_id, keypackage_ref);
        Err(MlsError::NostrMlsError("add_member_device not yet implemented - requires mdk-core API details".to_string()))
    }

    /// Makes the bot leave a group.
    ///
    /// # Arguments
    /// * `group_id` - The ID of the group to leave
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn leave_group(&self, group_id: &str) -> Result<(), MlsError> {
        // Stub implementation - actual implementation depends on mdk-core API
        // This is a placeholder to satisfy the TODO
        let _ = group_id;
        Err(MlsError::NostrMlsError("leave_group not yet implemented - requires mdk-core API details".to_string()))
    }

    /// Removes a member device from a group.
    ///
    /// # Arguments
    /// * `group_id` - The ID of the group
    /// * `member_pubkey` - The public key of the member to remove
    /// * `device_id` - The device ID to remove
    ///
    /// # Returns
    /// Result indicating success or failure
    pub async fn remove_member_device_from_group(
        &self,
        group_id: &str,
        member_pubkey: &str,
        device_id: &str,
    ) -> Result<(), MlsError> {
        // Stub implementation - actual implementation depends on mdk-core API
        // This is a placeholder to satisfy the TODO
        let _ = (group_id, member_pubkey, device_id);
        Err(MlsError::NostrMlsError("remove_member_device_from_group not yet implemented - requires mdk-core API details".to_string()))
    }

    /// Sends a message to a group.
    ///
    /// # Arguments
    /// * `group_id` - The ID of the group to send the message to
    /// * `message` - The message content to send
    ///
    /// # Returns
    /// Result containing the created event or an error
    pub async fn send_group_message(
        &self,
        group_id: &str,
        message: &str,
    ) -> Result<nostr_sdk::Event, MlsError> {
        // Stub implementation - actual implementation depends on mdk-core API
        // This is a placeholder to satisfy the TODO
        let _ = (group_id, message);
        Err(MlsError::NostrMlsError("send_group_message not yet implemented - requires mdk-core API details".to_string()))
    }

    /// Processes an incoming MLS event from a Nostr event JSON string.
    ///
    /// # Arguments
    /// * `event_json` - The JSON string of the Nostr event
    ///
    /// # Returns
    /// Result indicating whether the event was processed successfully
    pub async fn incoming_event(&self, event_json: &str) -> Result<bool, MlsError> {
        // Stub implementation - actual implementation depends on mdk-core API
        // This is a placeholder to satisfy the TODO
        let _ = event_json;
        Err(MlsError::NostrMlsError("incoming_event not yet implemented - requires mdk-core API details".to_string()))
    }

    /// Synchronizes group data from storage.
    ///
    /// # Arguments
    /// * `group_id` - The ID of the group to synchronize
    ///
    /// # Returns
    /// Result containing the group metadata or an error
    pub async fn sync_group_data(&self, group_id: &str) -> Result<MlsGroupMetadata, MlsError> {
        // Stub implementation - actual implementation depends on mdk-core API
        // This is a placeholder to satisfy the TODO
        let _ = group_id;
        Err(MlsError::NostrMlsError("sync_group_data not yet implemented - requires mdk-core API details".to_string()))
    }




}
