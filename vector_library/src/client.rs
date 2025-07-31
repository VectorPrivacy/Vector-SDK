use nostr_sdk::prelude::*;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

/// Configures and builds a vector client with the given keys and metadata.
///
/// This function sets up the client with proxy configuration for .onion relays,
/// adds default relays, and configures metadata.
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
/// A configured vector client.
pub async fn build_client(
    keys: Keys,
    name: String,
    display_name: String,
    about: String,
    picture: Url,
    banner: Url,
    nip05: String,
    lud16: String,
) -> Client {
    // Configure client to use proxy for .onion relays
    let addr = SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050));
    let connection = Connection::new()
        .proxy(addr) // Use `.embedded_tor()` instead to enable the embedded tor client (require `tor` feature)
        .target(ConnectionTarget::Onion);
    let opts = Options::new().connection(connection);

    // Create new client with custom options
    let client = Client::builder().signer(keys.clone()).opts(opts).build();

    // Add default relays
    client.add_relay("wss://jskitty.cat/nostr").await.unwrap();
    client.add_relay("wss://relay.damus.io").await.unwrap();
    client.add_relay("wss://auth.nostr1.com").await.unwrap();
    client.add_relay("wss://nostr.computingcache.com").await.unwrap();


    // Connect to relays
    client.connect().await;

    // Set up metadata
    let metadata = crate::metadata::create_metadata(
        name,
        display_name,
        about,
        picture,
        banner,
        nip05,
        lud16,
    );

    // Update metadata
    client.set_metadata(&metadata).await.unwrap();

    // Set up subscription for gift wrap events
    let subscription = crate::subscription::create_gift_wrap_subscription(keys.public_key());

    client.subscribe(subscription, None).await.unwrap();

    client
}