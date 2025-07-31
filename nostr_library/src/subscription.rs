use nostr_sdk::prelude::*;

/// Creates a subscription filter for gift wrap events.
///
/// This function sets up a filter to subscribe to gift wrap events for a specific public key.
///
/// # Arguments
///
/// * `pubkey` - The public key to filter events for.
///
/// # Returns
///
/// A configured Filter object for gift wrap events.
pub fn create_gift_wrap_subscription(pubkey: PublicKey) -> Filter {
    Filter::new()
        .pubkey(pubkey)
        .kind(Kind::GiftWrap)
        .limit(0) // Limit set to 0 to get only new events!
}