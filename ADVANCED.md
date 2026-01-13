# Advanced Documentation

This document covers advanced features and concepts in the Vector SDK.

## Table of Contents

- [Message Layer Security (MLS)](#message-layer-security-mls)
- [Typing Indicators](#typing-indicators)
- [Proxy Configuration](#proxy-configuration)
- [Error Handling](#error-handling)
- [Debugging](#debugging)
- [Logging Configuration](#logging-configuration)
- [Custom Metadata](#custom-metadata)
- [Group Management](#group-management)
- [File Handling](#file-handling)
- [Performance Considerations](#performance-considerations)

## Message Layer Security (MLS)

Vector SDK integrates with the [nostr-mls](https://github.com/nostr-protocol/nips/tree/master/nips) protocol for secure group messaging.

### Overview

MLS provides:
- End-to-end encryption for group messages
- Forward secrecy through ephemeral keys
- Efficient key management for groups
- Secure group membership changes

### Group Lifecycle

#### Creating a Group

```rust
use vector_sdk::VectorBot;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keys = Keys::generate();
    let bot = VectorBot::quick(keys).await;

    // Group creation is handled automatically when processing welcome events
    // See "Joining a Group" below
}
```

#### Joining a Group

```rust
use vector_sdk::VectorBot;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keys = Keys::generate();
    let bot = VectorBot::quick(keys).await;

    // Process a welcome event to join a group
    let welcome_event = UnsignedEvent::from_json("...")?;
    let group = bot.quick_join_group(welcome_event).await?;

    println!("Joined group: {:?}", group);
    Ok(())
}
```

#### Sending Group Messages

```rust
use vector_sdk::VectorBot;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keys = Keys::generate();
    let bot = VectorBot::quick(keys).await;

    // Get a group (after joining)
    let group_id = mdk_core::GroupId::from_hex("...")?;
    let group = bot.get_group(group_id).await?;

    // Send a message to the group
    group.send_group_message("Hello group!").await?;

    // Send a typing indicator
    group.send_group_typing_indicator().await;

    // Send a reaction
    group.send_group_reaction("event_id_hex".to_string(), "❤️".to_string()).await;

    Ok(())
}
```

#### Sending Files in Groups

```rust
use vector_sdk::{VectorBot, AttachmentFile, load_file};
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keys = Keys::generate();
    let bot = VectorBot::quick(keys).await;

    let group_id = mdk_core::GroupId::from_hex("...")?;
    let group = bot.get_group(group_id).await?;

    // Load and send a file
    let attachment = load_file("path/to/file.png")?;
    group.send_group_attachment(Some(attachment)).await?;

    Ok(())
}
```

### Group Metadata

Groups store metadata including:
- `group_id`: Wire identifier used on relays
- `engine_group_id`: Internal engine identifier
- `creator_pubkey`: Public key of group creator
- `name`: Group display name
- `avatar_ref`: Reference to group avatar
- `created_at`: Unix timestamp of creation
- `updated_at`: Unix timestamp of last update
- `evicted`: Flag indicating if user was evicted

## Typing Indicators

Typing indicators provide real-time feedback about message composition.

### Implementation Details

- **Protocol**: NIP-40 (Application-Specific Data)
- **Kind**: `Kind::ApplicationSpecificData` (31999)
- **Content**: String "typing"
- **Expiration**: 30 seconds from creation
- **Tags**:
  - `d` tag with value "vector" for namespace
  - `ms` tag with millisecond precision timestamp
  - `expiration` tag for relay cleanup

### Usage

#### Direct Messages

```rust
use vector_sdk::VectorBot;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keys = Keys::generate();
    let bot = VectorBot::quick(keys).await;

    let recipient = PublicKey::from_bech32("npub1...")?;
    let chat = bot.get_chat(recipient).await;

    // Send typing indicator
    chat.send_typing_indicator().await;

    // Simulate work
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Send actual message
    chat.send_private_message("Here's my response!").await;

    Ok(())
}
```

#### Groups

```rust
use vector_sdk::VectorBot;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keys = Keys::generate();
    let bot = VectorBot::quick(keys).await;

    let group_id = mdk_core::GroupId::from_hex("...")?;
    let group = bot.get_group(group_id).await?;

    // Send typing indicator to group
    group.send_group_typing_indicator().await;

    // Simulate work
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Send actual message
    group.send_group_message("Here's my response!").await?;

    Ok(())
}
```

## Proxy Configuration

Vector SDK supports proxy configuration for .onion relays.

### Configuration Options

```rust
use vector_sdk::client::ClientConfig;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4};

let config = ClientConfig {
    proxy_addr: Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050))),
    default_relays: vec![
        "wss://jskitty.cat/nostr".to_string(),
        "wss://relay.damus.io".to_string(),
    ],
};
```

### Using Tor

To use the embedded Tor client instead of a SOCKS proxy:

```rust
use nostr_sdk::{Connection, ConnectionTarget, Options};
use vector_sdk::VectorBot;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keys = Keys::generate();

    // Configure client with embedded Tor
    let connection = Connection::new()
        .embedded_tor() // Enable embedded Tor client
        .target(ConnectionTarget::Onion);
    let opts = Options::new().connection(connection);

    let mut client = Client::builder()
        .signer(keys.clone())
        .opts(opts)
        .build();

    // Add relays and continue with VectorBot initialization
    // ...
}
```

## Error Handling

Vector SDK provides comprehensive error handling through the `VectorBotError` enum.

### Error Types

```rust
pub enum VectorBotError {
    Mls(mls::MlsError),
    Crypto(crate::crypto::CryptoError),
    Upload(crate::upload::UploadError),
    UrlParse(url::ParseError),
    Io(std::io::Error),
    Nostr(String),
    SerdeJson(serde_json::Error),
    InvalidInput(String),
    Network(String),
    Storage(String),
    Metadata(crate::metadata::MetadataError),
    Subscription(crate::subscription::SubscriptionError),
}
```

### Handling Errors

```rust
use vector_sdk::{VectorBot, VectorBotError};
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() {
    let keys = Keys::generate();

    match VectorBot::quick(keys).await {
        Ok(bot) => {
            // Success path
        }
        Err(VectorBotError::Nostr(e)) => {
            eprintln!("Nostr error: {}", e);
        }
        Err(VectorBotError::Io(e)) => {
            eprintln!("IO error: {}", e);
        }
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
        }
    }
}
```

### Converting Errors

```rust
use vector_sdk::VectorBotError;
use std::fmt;

// Convert string errors
let err = VectorBotError::from("error message".to_string());
let err = VectorBotError::from("error message");

// Convert from other error types
let io_err = std::io::Error::new(std::io::ErrorKind::Other, "test");
let vector_err: VectorBotError = io_err.into();
```

## Debugging

### Logging Setup

Configure logging in your application:

```rust
use env_logger;
use log::LevelFilter;

fn main() {
    // Initialize logging
    env_logger::Builder::new()
        .filter_level(LevelFilter::Debug)
        .init();

    // Your application code
}
```

### Log Levels

- **Error**: Critical errors that need attention
- **Warn**: Potential issues or deprecated features
- **Info**: Important operational messages
- **Debug**: Detailed debugging information
- **Trace**: Very detailed tracing information

### Debugging Tips

1. **Enable verbose logging**:
   ```bash
   RUST_LOG=debug cargo run
   ```

2. **Check relay connections**:
   ```rust
   use log::info;
   use nostr_sdk::prelude::*;

   // After creating client
   info!("Connected to relays: {:?}", client.relays());
   ```

3. **Inspect events**:
   ```rust
   use log::debug;
   use nostr_sdk::prelude::*;

   // When receiving events
   debug!("Received event: {:?}", event);
   ```

4. **Monitor uploads**:
   ```rust
   use log::info;

   // Set up progress callback
   let progress_callback = std::sync::Arc::new(move |percentage, bytes| {
       if let Some(pct) = percentage {
           info!("Upload progress: {}%", pct);
       }
       Ok(())
   });
   ```

## Logging Configuration

### Custom Log Format

```rust
use log::{Level, Record};
use std::io::Write;

struct CustomLogger;

impl log::Log for CustomLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let _ = writeln!(
                std::io::stderr(),
                "[{}] {} - {}",
                record.level(),
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

fn main() {
    log::set_boxed_logger(Box::new(CustomLogger)).unwrap();
    log::set_max_level(log::LevelFilter::Debug);
}
```

### Structured Logging

For applications that need structured logging:

```rust
use serde_json;
use log::Record;

struct JsonLogger;

impl log::Log for JsonLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= log::max_level()
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let log_entry = serde_json::json!({
                "timestamp": chrono::Utc::now().to_rfc3339(),
                "level": record.level().to_string().to_lowercase(),
                "message": record.args().to_string(),
                "target": record.target(),
            });
            println!("{}", log_entry);
        }
    }

    fn flush(&self) {}
}
```

## Custom Metadata

Vector SDK supports custom metadata fields through the builder pattern.

### Using the Metadata Builder

```rust
use vector_sdk::metadata::{MetadataConfig, MetadataConfigBuilder};
use url::Url;

let metadata = MetadataConfigBuilder::new()
    .name("My Bot".to_string())
    .display_name("my_bot".to_string())
    .about("A helpful bot".to_string())
    .picture(Url::parse("https://example.com/avatar.png")?)
    .banner(Url::parse("https://example.com/banner.png")?)
    .nip05("bot@example.com".to_string())
    .lud16("bot@example.com".to_string())
    .build();

println!("Metadata: {:?}", metadata);
```

### Adding Custom Fields

```rust
use nostr_sdk::prelude::*;

let mut metadata = Metadata::new()
    .name("My Bot")
    .display_name("my_bot")
    .about("A helpful bot")
    .custom_field("custom_field", "custom_value")
    .custom_field("version", "1.0.0")
    .custom_field("website", "https://example.com");
```

## Group Management

### Checking Group Messages

```rust
use vector_sdk::VectorBot;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keys = Keys::generate();
    let bot = VectorBot::quick(keys).await;

    let group_id = mdk_core::GroupId::from_hex("...")?;
    let group = bot.get_group(group_id).await?;

    // Check all messages in the group
    group.check_group_messages().await?;

    // Get a specific message
    let message_id = EventId::from_hex("...")?;
    group.get_message(&message_id).await?;

    Ok(())
}
```

### Processing Incoming Messages

```rust
use vector_sdk::VectorBot;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let keys = Keys::generate();
    let bot = VectorBot::quick(keys).await;

    // Subscribe to MLS group messages
    let filter = Filter::new()
        .kind(Kind::MlsGroupMessage)
        .limit(0);

    bot.client.subscribe(filter, None).await?;

    // Process events
    while let Some(event) = bot.client.next_incoming_message().await {
        match event {
            RelayPoolNotification::Message { message, .. } => {
                if let Ok(processed_msg) = bot.process_group_message(&message.event).await {
                    println!("Received message: {:?}", processed_msg);
                }
            }
            _ => {}
        }
    }

    Ok(())
}
```

## File Handling

### File Type Detection

Vector SDK automatically detects file types from bytes:

```rust
use vector_sdk::AttachmentFile;

let bytes = std::fs::read("unknown_file")?;
let attachment = AttachmentFile::from_bytes(bytes);

println!("Detected extension: {}", attachment.extension);
```

### Image Metadata

For image files, additional metadata is extracted:

```rust
use vector_sdk::{AttachmentFile, load_file};

let attachment = load_file("image.png")?;

if let Some(img_meta) = attachment.img_meta {
    println!("Blurhash: {}", img_meta.blurhash);
    println!("Dimensions: {}x{}", img_meta.width, img_meta.height);
}
```

### File Hashing

Files are hashed using SHA-256 for integrity verification:

```rust
use vector_sdk::calculate_file_hash;

let data = b"Hello, world!";
let hash = calculate_file_hash(data);

println!("File hash: {}", hash);
```

## Performance Considerations

### Upload Performance

- **Chunk Size**: Default 64KB chunks for streaming uploads
- **Retry Strategy**: Configurable retry count and spacing
- **Progress Tracking**: Periodic progress callbacks (every 100ms)
- **Stall Detection**: Detects stalled uploads after 20 seconds

### Configuration Options

```rust
use vector_sdk::upload::{UploadConfig, UploadParams};

let upload_config = UploadConfig {
    connect_timeout: std::time::Duration::from_secs(5),
    pool_idle_timeout: std::time::Duration::from_secs(90),
    pool_max_idle_per_host: 2,
    stall_threshold: 200, // 20 seconds
};

let upload_params = UploadParams {
    retry_count: 3,
    retry_spacing: std::time::Duration::from_secs(2),
    chunk_size: 64 * 1024, // 64 KB
};
```

### Memory Management

- **Streaming Uploads**: Files are streamed in chunks to minimize memory usage
- **Encryption**: Data is encrypted in-place when possible
- **Cleanup**: Temporary data is cleared after processing

### Concurrent Operations

Vector SDK is designed to work with Tokio's async runtime:
- Multiple uploads can run concurrently
- Messages can be sent while uploads are in progress
- Group operations are non-blocking
