use log::warn;
use nostr_sdk::prelude::*;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

/// Configuration options for the vector client.
pub struct ClientConfig {
    /// The address of the proxy server for .onion relays.
    pub proxy_addr: Option<SocketAddr>,
    /// A list of default relays to connect to.
    pub default_relays: Vec<String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            proxy_addr: Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050))),
            default_relays: vec![
                "wss://jskitty.cat/nostr".to_string(),
                "wss://relay.damus.io".to_string(),
                "wss://auth.nostr1.com".to_string(),
                "wss://nostr.computingcache.com".to_string(),
            ],
        }
    }
}

/// Configures and builds a vector client with the given keys and metadata.
///
/// This function sets up the client with optional proxy configuration for .onion relays,
/// adds configurable relays, and configures metadata.
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
/// * `config` - Optional client configuration.
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
    config: Option<ClientConfig>,
) -> Client {
    let config = config.unwrap_or_default();

    // Create new client with default options
    let mut client = Client::builder().signer(keys.clone()).build();

    // Configure proxy if provided
    if let Some(proxy_addr) = config.proxy_addr {
        let connection = Connection::new()
            .proxy(proxy_addr) // Use `.embedded_tor()` instead to enable the embedded tor client (require `tor` feature)
            .target(ConnectionTarget::Onion);
        let opts = Options::new().connection(connection);
        client = Client::builder().signer(keys.clone()).opts(opts).build();
    }

    // Add default relays
    for relay in &config.default_relays {
        if let Err(e) = client.add_relay(relay).await {
            warn!("Failed to add relay {relay}: {e:?}");
        }
    }

    // Connect to relays
    client.connect().await;

    // Set up metadata
    let metadata = crate::metadata::create_metadata(
        name,
        display_name,
        about,
        Some(picture),
        Some(banner),
        Some(nip05),
        Some(lud16),
    );

    // Update metadata
    let _ = client.set_metadata(&metadata).await;

    // Set up subscription for gift wrap events
    let subscription =
        crate::subscription::create_gift_wrap_subscription(keys.public_key(), None, None).unwrap();

    let _ = client.subscribe(subscription, None).await;

    client
}
