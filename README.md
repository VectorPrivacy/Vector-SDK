# Vector Bot Library

## Table of Contents

- [Overview](#overview)
- [Features](#features)
- [Documentation](#documentation)
- [Architecture](#architecture)
- [Installation](#installation)
- [Usage](#usage)
  - [Sending a Text Message](#sending-a-text-message)
  - [Sending an Image](#sending-an-image)
  - [Creating an Attachment from in-memory bytes](#creating-an-attachment-from-in-memory-bytes)
  - [Typing indicators](#typing-indicators)
- [Components](#components)
- [Dependencies](#dependencies)
- [License](#license)

## Overview

The Vector Bot Library is a Rust-based library for creating and managing Vector Bots that can send and receive private messages using the Nostr protocol. This library provides a structured and modular approach to building bots with configurable metadata and client settings.

## Features

- Create vector bots with customizable metadata
- Send and receive private messages and files
- Handle notifications for gift wrap events
- Configure proxy settings for .onion relays
- Add and manage relays
- Modular architecture for easy extension and maintenance
- Message Layer Security (MLS) for group messaging
- Typing indicators for real-time feedback
- Reaction support for messages
- File uploads with progress tracking
- Automatic failover for media servers

### MLS Implementation Status

The Vector SDK includes **Message Layer Security (MLS)** support for group messaging. The following MLS features are currently implemented and ready for use:

✅ **Implemented Features:**
- Group joining via welcome events
- Group message sending and processing
- Group typing indicators
- Group file attachments
- Group reactions
- Persistent SQLite-backed storage for group state

⚠️ **Placeholder Functions (Not Yet Implemented):**
The following MLS functions exist as stubs and will be implemented in future versions when needed:
- `create_group()` - Group creation functionality
- `add_member_device()` - Adding members to groups
- `leave_group()` - Leaving groups
- `remove_member_device_from_group()` - Removing members
- `send_group_message()` - Direct group message sending (use `Group::send_group_message()` instead)

These placeholder functions are available in the API but will return errors if called. They are included to provide a complete API surface for future expansion.

## Documentation

For comprehensive documentation, see:

- **[CONTRIBUTING.md](CONTRIBUTING.md)** - Development guidelines and contribution process
- **[SECURITY.md](SECURITY.md)** - Security features, best practices, and threat model
- **[ADVANCED.md](ADVANCED.md)** - Advanced features including MLS, typing indicators, and debugging
- **[TROUBLESHOOTING.md](TROUBLESHOOTING.md)** - Common issues and solutions
- **[CHANGELOG.md](CHANGELOG.md)** - Release history and changes

## Examples

For practical examples and working code demonstrations, see the [Vector-SDK-Example](https://github.com/Luke-Larsen/Vector-SDK-Example) repository, which contains:
- Basic bot setup examples
- Direct messaging implementations
- Group messaging examples
- File handling demonstrations
- Advanced use cases
- Complete working applications

## Architecture

The library is organized into several modules, each responsible for a specific aspect of the bot's functionality:

1. **VectorBot**: The main struct representing the bot, containing metadata and client configuration.
2. **Channel**: A struct for managing communication with specific recipients.
3. **Client**: Functions for building and configuring the Nostr client.
4. **Metadata**: Functions for creating and managing bot metadata.
5. **Subscription**: Functions for setting up event subscriptions.
6. **Crypto**: Functions for encryption and decryption.
7. **Upload**: Functions for handling file uploads.
8. **MLS**: Message Layer Security for group messaging.
9. **Blossom**: Media server integration for file uploads.

### High-Level Architecture

```
+---------------------+
|     VectorBot       |
|---------------------|
| - keys              |
| - name              |
| - display_name      |
| - about             |
| - picture           |
| - banner            |
| - nip05             |
| - lud16             |
| - client            |
| - device_mdk        |
|---------------------|
| + quick()           |
| + new()             |
| + get_chat()        |
| + get_group()       |
| + checkout_group()  |
| + process_group_message() |
+---------------------+
          |
          v
+---------------------+
|      Channel        |
|---------------------|
| - recipient         |
| - base_bot          |
|---------------------|
| + new()             |
| + send_private_message()|
| + send_private_file()|
| + send_typing_indicator()|
| + send_reaction()   |
+---------------------+

+---------------------+
|      Group         |
|---------------------|
| - group             |
| - base_bot          |
|---------------------|
| + new()             |
| + get_message()     |
| + check_group_messages()|
| + send_group_message()|
| + send_group_attachment()|
| + send_group_typing_indicator()|
| + send_group_reaction()|
+---------------------+

+---------------------+
|      Client        |
|---------------------|
| + build_client()   |
+---------------------+

+---------------------+
|     Metadata       |
|---------------------|
| + create_metadata()|
| + MetadataConfigBuilder |
+---------------------+

+---------------------+
|   Subscription     |
|---------------------|
| + create_gift_wrap_|
|   subscription()    |
+---------------------+

+---------------------+
|      Crypto        |
|---------------------|
| + generate_encryption_params() |
| + encrypt_data()   |
+---------------------+

+---------------------+
|      Upload        |
|---------------------|
| + upload_data_with_progress() |
+---------------------+

+---------------------+
|      Blossom       |
|---------------------|
| + upload_blob_with_progress_and_failover() |
+---------------------+

+---------------------+
|        MLS         |
|---------------------|
| + MlsGroup          |
| + new_persistent() |
| + engine()          |
+---------------------+
```

For more detailed information about the architecture and advanced features, see [ADVANCED.md](ADVANCED.md).

## Installation

To use the Vector Bot Library, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
vector_sdk = "0.3.0"
```

## Usage

### Sending a Text Message

```rust
use vector_sdk::VectorBot;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Generate new random keys
    let keys = Keys::generate();

    // Create a new VectorBot with default metadata
    let bot = VectorBot::quick(keys).await;

    // Get a chat channel for a specific public key
    let chat_npub = PublicKey::from_bech32("npub1example...")?;
    let chat = bot.get_chat(chat_npub).await;

    // Send a private message
    let success = chat.send_private_message("Hello, world!").await;
    println!("Message sent: {}", success);

    Ok(())
}
```

### Sending an Image

```rust
use vector_sdk::{VectorBot, AttachmentFile, load_file};
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate new random keys
    let keys = Keys::generate();

    // Create a new VectorBot with default metadata
    let bot = VectorBot::quick(keys).await;

    // Get a chat channel for a specific public key
    let chat_npub = PublicKey::from_bech32("npub1example...")?;
    let chat = bot.get_chat(chat_npub).await;

    // Load a file from disk in one line
    let attachment: AttachmentFile = load_file("path/to/your/image.png")?;

    // Send the image
    let success = chat.send_private_file(Some(attachment)).await;
    println!("Image sent: {}", success);

    Ok(())
}
```

Alternatively, you can use the inherent constructor:

```rust
let attachment = AttachmentFile::from_path("path/to/your/image.png")?;
```

### Creating an Attachment from in-memory bytes

If you already loaded bytes in memory (e.g., `fs::read`), construct the attachment in a single line. The extension is inferred via magic-number sniffing and falls back to "bin" when unknown.

```rust
use vector_sdk::AttachmentFile;
use std::fs;

let bytes = fs::read("path/to/your/image.png")?;
let attachment = AttachmentFile::from_bytes(bytes);
```

### Typing indicators

Typing indicators provide real-time feedback to recipients that a bot is composing a message. This is useful when a bot needs to retrieve information or is "thinking" before responding.

```rust
use vector_sdk::VectorBot;
use nostr_sdk::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Generate new random keys
    let keys = Keys::generate();

    // Create a new VectorBot with default metadata
    let bot = VectorBot::quick(keys).await;

    // Get a chat channel for a specific public key
    let chat_npub = PublicKey::from_bech32("npub1example...")?;
    let chat = bot.get_chat(chat_npub).await;

    // Send typing indicator (shows "recipient is typing...")
    chat.send_typing_indicator().await;

    // Simulate work (e.g., fetching data, processing)
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    // Send the actual message
    let success = chat.send_private_message("Here's my response!").await;
    println!("Message sent: {}", success);

    Ok(())
}
```

For more information about typing indicators and their implementation, see [ADVANCED.md](ADVANCED.md#typing-indicators).



## Components

### VectorBot

The `VectorBot` struct represents a bot with configured metadata and client settings. It provides methods for creating new bots, getting chat channels, and sending messages.

### Channel

The `Channel` struct represents a communication channel with a specific recipient. It provides methods for sending private messages.

### Client

The `Client` module contains functions for building and configuring the Nostr client, including setting up proxy configurations, adding relays, and managing metadata.

### Metadata

The `Metadata` module contains functions for creating and managing bot metadata, including name, display name, about, picture, banner, NIP05 identifier, and LUD16 payment pointer.

### Subscription

The `Subscription` module contains functions for setting up event subscriptions, such as gift wrap events.

### Crypto

The `Crypto` module provides functions for generating encryption parameters and encrypting data using AES-256-GCM.

### Upload

The `Upload` module provides functions for uploading data to a NIP-96 server with progress tracking.

## Dependencies

- `nostr_sdk`: The Nostr SDK for Rust, providing the core functionality for interacting with the Nostr protocol.
- `url`: For handling URLs in metadata.
- `tokio`: For asynchronous runtime.
- `aes`: For AES encryption.
- `aes_gcm`: For AES-GCM encryption.
- `reqwest`: For HTTP requests.
- `sha2`: For SHA-256 hashing.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
