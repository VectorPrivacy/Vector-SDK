use ::url::Url;
use nostr_sdk::prelude::*;
// Removed unused import
use std::fmt;

/// Errors that can occur during metadata operations
#[derive(Debug)]
pub enum MetadataError {
    /// Invalid metadata format
    InvalidFormat(String),
    /// Missing required field
    MissingField(String),
}

impl fmt::Display for MetadataError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetadataError::InvalidFormat(msg) => write!(f, "Invalid metadata format: {msg}"),
            MetadataError::MissingField(field) => write!(f, "Missing required field: {field}"),
        }
    }
}

impl std::error::Error for MetadataError {}

/// Configuration options for user metadata.
///
/// This struct contains all the fields that can be configured in user metadata.
/// It provides a flexible way to build metadata with optional fields.
#[derive(Debug, Clone)]
pub struct MetadataConfig {
    /// The name of the user.
    pub name: String,
    /// The display name of the user.
    pub display_name: String,
    /// A brief description about the user.
    pub about: String,
    /// The URL of the user's profile picture.
    pub picture: Option<Url>,
    /// The URL of the user's banner.
    pub banner: Option<Url>,
    /// The NIP05 identifier.
    pub nip05: Option<String>,
    /// The LUD16 payment pointer.
    pub lud16: Option<String>,
}

impl MetadataConfig {
    /// Creates a new MetadataConfig builder.
    ///
    /// # Returns
    ///
    /// A MetadataConfigBuilder for configuring metadata.
    pub fn builder() -> MetadataConfigBuilder {
        MetadataConfigBuilder::new()
    }

    /// Creates metadata from the configuration.
    ///
    /// This function builds a Metadata object from the configured fields.
    ///
    /// # Returns
    ///
    /// A configured Metadata object.
    pub fn build(&self) -> Metadata {
        let mut metadata = Metadata::new()
            .name(&self.name)
            .display_name(&self.display_name)
            .about(&self.about);

        if let Some(ref picture) = self.picture {
            metadata = metadata.picture(picture.clone());
        }

        if let Some(ref banner) = self.banner {
            metadata = metadata.banner(banner.clone());
        }

        if let Some(ref nip05) = self.nip05 {
            metadata = metadata.nip05(nip05.clone());
        }

        if let Some(ref lud16) = self.lud16 {
            metadata = metadata.lud16(lud16.clone());
        }

        metadata
    }
}

/// Builder for MetadataConfig.
///
/// This struct provides a fluent interface for configuring metadata.
#[derive(Debug, Clone)]
pub struct MetadataConfigBuilder {
    config: MetadataConfig,
}

impl MetadataConfigBuilder {
    /// Creates a new MetadataConfigBuilder.
    ///
    /// # Returns
    ///
    /// A new MetadataConfigBuilder.
    pub fn new() -> Self {
        Self {
            config: MetadataConfig {
                name: String::new(),
                display_name: String::new(),
                about: String::new(),
                picture: None,
                banner: None,
                nip05: None,
                lud16: None,
            },
        }
    }

    /// Sets the name field.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the user.
    ///
    /// # Returns
    ///
    /// The builder for method chaining.
    pub fn name(mut self, name: String) -> Self {
        self.config.name = name;
        self
    }

    /// Sets the display name field.
    ///
    /// # Arguments
    ///
    /// * `display_name` - The display name of the user.
    ///
    /// # Returns
    ///
    /// The builder for method chaining.
    pub fn display_name(mut self, display_name: String) -> Self {
        self.config.display_name = display_name;
        self
    }

    /// Sets the about field.
    ///
    /// # Arguments
    ///
    /// * `about` - A brief description about the user.
    ///
    /// # Returns
    ///
    /// The builder for method chaining.
    pub fn about(mut self, about: String) -> Self {
        self.config.about = about;
        self
    }

    /// Sets the picture field.
    ///
    /// # Arguments
    ///
    /// * `picture` - The URL of the user's profile picture.
    ///
    /// # Returns
    ///
    /// The builder for method chaining.
    pub fn picture(mut self, picture: Url) -> Self {
        self.config.picture = Some(picture);
        self
    }

    /// Sets the banner field.
    ///
    /// # Arguments
    ///
    /// * `banner` - The URL of the user's banner.
    ///
    /// # Returns
    ///
    /// The builder for method chaining.
    pub fn banner(mut self, banner: Url) -> Self {
        self.config.banner = Some(banner);
        self
    }

    /// Sets the NIP05 identifier field.
    ///
    /// # Arguments
    ///
    /// * `nip05` - The NIP05 identifier.
    ///
    /// # Returns
    ///
    /// The builder for method chaining.
    pub fn nip05(mut self, nip05: String) -> Self {
        self.config.nip05 = Some(nip05);
        self
    }

    /// Sets the LUD16 payment pointer field.
    ///
    /// # Arguments
    ///
    /// * `lud16` - The LUD16 payment pointer.
    ///
    /// # Returns
    ///
    /// The builder for method chaining.
    pub fn lud16(mut self, lud16: String) -> Self {
        self.config.lud16 = Some(lud16);
        self
    }

    /// Builds the Metadata object.
    ///
    /// # Returns
    ///
    /// A configured Metadata object.
    pub fn build(self) -> Metadata {
        self.config.build()
    }
}

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
    picture: Option<Url>,
    banner: Option<Url>,
    nip05: Option<String>,
    lud16: Option<String>,
) -> Metadata {
    MetadataConfig {
        name,
        display_name,
        about,
        picture,
        banner,
        nip05,
        lud16,
    }
    .build()
}
