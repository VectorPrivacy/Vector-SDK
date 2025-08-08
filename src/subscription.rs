use nostr_sdk::prelude::*;
// Removed unused import
use std::fmt;

/// Errors that can occur during subscription operations
#[derive(Debug)]
pub enum SubscriptionError {
    /// Invalid filter configuration
    InvalidFilter(String),
}

impl fmt::Display for SubscriptionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubscriptionError::InvalidFilter(msg) => write!(f, "Invalid filter configuration: {}", msg),
        }
    }
}

impl std::error::Error for SubscriptionError {}

/// Configuration options for subscriptions
#[derive(Debug, Clone)]
pub struct SubscriptionConfig {
    /// The public key to filter events for
    pub pubkey: PublicKey,
    /// The kind of events to filter
    pub kind: Kind,
    /// The maximum number of events to return (0 means no limit)
    pub limit: u64,
}

impl Default for SubscriptionConfig {
    fn default() -> Self {
        Self {
            pubkey: PublicKey::from_bech32("npub1").expect("Invalid public key"),
            kind: Kind::GiftWrap,
            limit: 0,
        }
    }
}

/// Creates a subscription filter for gift wrap events.
///
/// This function sets up a filter to subscribe to gift wrap events for a specific public key.
///
/// # Arguments
///
/// * `pubkey` - The public key to filter events for.
/// * `kind` - The kind of events to filter (default: Kind::GiftWrap).
/// * `limit` - The maximum number of events to return (default: 0, meaning no limit).
///
/// # Returns
///
/// A configured Filter object for gift wrap events.
///
/// # Errors
///
/// Returns a SubscriptionError if the filter configuration is invalid.
pub fn create_gift_wrap_subscription(
    pubkey: PublicKey,
    kind: Option<Kind>,
    limit: Option<u64>,
) -> Result<Filter, SubscriptionError> {
    let kind = kind.unwrap_or(Kind::GiftWrap);
    let limit = limit.unwrap_or(0);

    if limit > 1000 {
        return Err(SubscriptionError::InvalidFilter("Limit exceeds maximum allowed value (1000)".into()));
    }

    Ok(Filter::new()
        .pubkey(pubkey)
        .kind(kind)
        .limit(limit.try_into().unwrap()))
}