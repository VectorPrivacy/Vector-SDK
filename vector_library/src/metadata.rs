use nostr_sdk::prelude::*;
use url::Url;

/// Creates and configures metadata for a vector user.
///
/// This function builds a Metadata object with the provided information.
///
/// # Arguments
///
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
/// A configured Metadata object.
pub fn create_metadata(
    name: String,
    display_name: String,
    about: String,
    picture: Url,
    banner: Url,
    nip05: String,
    lud16: String,
) -> Metadata {
    Metadata::new()
        .name(name)
        .display_name(display_name)
        .about(about)
        .picture(picture)
        .banner(banner)
        .nip05(nip05)
        .lud16(lud16)
}