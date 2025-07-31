# Vector Bot Library

## Overview

The Vector Bot Library is a Rust-based library for creating and managing vector bots that can send and receive private messages using the Nostr protocol. This library provides a structured and modular approach to building bots with configurable metadata and client settings.

## Features

- Create vector bots with customizable metadata
- Send and receive private messages
- Handle notifications for gift wrap events
- Configure proxy settings for .onion relays
- Add and manage relays
- Modular architecture for easy extension and maintenance

## Architecture

The library is organized into several modules, each responsible for a specific aspect of the bot's functionality:

1. **VectorBot**: The main struct representing the bot, containing metadata and client configuration.
2. **Channel**: A struct for managing communication with specific recipients.
3. **Client**: Functions for building and configuring the Nostr client.
4. **Metadata**: Functions for creating and managing bot metadata.
5. **Subscription**: Functions for setting up event subscriptions.

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
|---------------------|
| + quick()           |
| + new()             |
| + get_chat()        |
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
| + send_private_msg()|
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
+---------------------+

+---------------------+
|   Subscription     |
|---------------------|
| + create_gift_wrap_|
|   subscription()    |
+---------------------+
```

## Installation

To use the Vector Bot Library, add it as a dependency in your `Cargo.toml`:

```toml
[dependencies]
vector_library = { path = "path/to/vector_library" }
```

## Usage

```rust
use vector_library::VectorBot;
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

## Dependencies

- `nostr_sdk`: The Nostr SDK for Rust, providing the core functionality for interacting with the Nostr protocol.
- `url`: For handling URLs in metadata.
- `tokio`: For asynchronous runtime.

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.