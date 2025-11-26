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

/// Keypackage index entry stored in "mls_keypackage_index"
#[derive(Debug, Clone, Serialize, Deserialize)]
struct KeyPackageIndexEntry {
    owner_pubkey: String,
    device_id: String,
    keypackage_ref: String,
    fetched_at: u64,
    expires_at: u64,
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
    
    pub async fn create_group(){
        // TODO: Create a MLS group
    }
    
    pub async fn add_member_device(){
        // TODO: Add user to MLS group
    }

    pub async fn leave_group(){
        // TODO: Make the bot leave a group
    }

    pub async fn remove_member_device_from_group(){
        // TODO: removes a member device from the group
    }

    pub async fn send_group_message(){
        // TODO: send a message in the group
    }

    pub async fn incoming_event(&self, event_json: &str) -> Result<bool, MlsError> {
        // TODO: Parse nostr event JSON
        // TODO: Extract MLS ciphertext from event
        // TODO: Process through nostr-mls (handles welcome, commit, application messages)
        // TODO: Store any resulting messages in "mls_messages_{group_id}"
        // TODO: Update "mls_event_cursors" with event ID and timestamp
        
        // Stub implementation

        println!("Incoming Event: {:#?}", event_json);
        let _ = event_json;
        Ok(false)
    }

    pub async fn sync_group_data(&self, group_id: &str){
        // TODO: get all group data from the last message
    }




}