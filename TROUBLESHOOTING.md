# Troubleshooting Guide

This guide provides solutions to common issues and debugging techniques for Vector SDK.

## Table of Contents

- [Common Issues](#common-issues)
- [Connection Problems](#connection-problems)
- [Encryption Errors](#encryption-errors)
- [File Upload Issues](#file-upload-issues)
- [Group Messaging Problems](#group-messaging-problems)
- [Logging and Debugging](#logging-and-debugging)
- [Performance Issues](#performance-issues)
- [FAQ](#faq)

## Common Issues

### Issue: Bot fails to connect to relays

**Symptoms:**
- Connection timeouts
- No events received
- `Failed to add relay` errors

**Solutions:**
1. Check your internet connection
2. Verify relay URLs are correct and accessible
3. Try different relays:
   ```rust
   let config = ClientConfig {
       default_relays: vec![
           "wss://nos.lol".to_string(),
           "wss://relay.damus.io".to_string(),
           "wss://purplepag.es".to_string(),
       ],
       ..Default::default()
   };
   ```
4. Check if relays are online: https://nostr.watch

### Issue: Messages not being received

**Symptoms:**
- Sent messages don't appear for recipient
- No incoming messages
- Subscription not working

**Solutions:**
1. Verify you're using the correct public key:
   ```rust
   println!("Public key: {}", keys.public_key());
   ```
2. Check subscription filters:
   ```rust
   let filter = Filter::new()
       .pubkey(recipient_pubkey)
       .kind(Kind::GiftWrap)
       .limit(100);
   ```
3. Ensure relays support the required NIPs (NIP-59 for gift wrap)

### Issue: Encryption/decryption failures

**Symptoms:**
- `AesGcmError` exceptions
- Failed to decrypt messages
- Invalid key or nonce errors

**Solutions:**
1. Verify encryption parameters:
   ```rust
   let params = crypto::generate_encryption_params()?;
   println!("Key: {}, Nonce: {}", params.key, params.nonce);
   ```
2. Check that keys and nonces are properly transmitted
3. Ensure no corruption in transmitted data
4. Verify file integrity with SHA-256 hash

## Connection Problems

### Relay Connection Issues

**Error:** `Connection failed` or `WebSocket error`

**Debugging Steps:**
1. Test relay connectivity manually:
   ```bash
   curl -v https://jskitty.cat/nostr
   ```
2. Check firewall settings
3. Verify proxy configuration if using SOCKS proxy
4. Try adding more relays for redundancy

**Solution:**
```rust
// Add multiple relays for redundancy
let config = ClientConfig {
    default_relays: vec![
        "wss://nos.lol".to_string(),
        "wss://relay.damus.io".to_string(),
        "wss://purplepag.es".to_string(),
        "wss://nostr.wine".to_string(),
    ],
    ..Default::default()
};
```

### Tor/Onion Relay Issues

**Error:** `Failed to connect to onion relay`

**Debugging Steps:**
1. Verify Tor is running:
   ```bash
   systemctl status tor
   ```
2. Check Tor control port is accessible
3. Verify proxy address is correct:
   ```rust
   let config = ClientConfig {
       proxy_addr: Some(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 9050))),
       ..Default::default()
   };
   ```

**Solution:**
```rust
// Use embedded Tor instead of SOCKS proxy
let connection = Connection::new()
    .embedded_tor()
    .target(ConnectionTarget::Onion);
let opts = Options::new().connection(connection);
```

## Encryption Errors

### AES-GCM Errors

**Error:** `AesGcmError: invalid nonce size` or `invalid key size`

**Causes:**
- Incorrect key size (must be 32 bytes for AES-256)
- Incorrect nonce size (must be 16 bytes)
- Corrupted key or nonce during transmission

**Solution:**
```rust
// Always generate keys properly
let params = crypto::generate_encryption_params()?;
assert_eq!(params.key.len(), 64); // 32 bytes in hex
assert_eq!(params.nonce.len(), 32); // 16 bytes in hex
```

### Decryption Failures

**Error:** `Failed to decrypt` or `authentication failed`

**Causes:**
- Wrong key or nonce used
- Message tampered with in transit
- Authentication tag mismatch

**Debugging Steps:**
1. Verify the correct key and nonce are being used
2. Check message integrity with SHA-256 hash
3. Ensure no modification occurred during transmission

**Solution:**
```rust
// Verify file integrity before decryption
let file_hash = calculate_file_hash(&encrypted_data);
println!("File hash: {}", file_hash);
// Compare with expected hash
```

## File Upload Issues

### Upload Failures

**Error:** `Upload failed` or `All Blossom servers failed`

**Causes:**
- Network connectivity issues
- Server temporarily unavailable
- File too large
- Invalid MIME type

**Debugging Steps:**
1. Check network connection
2. Verify Blossom servers are online
3. Check file size:
   ```rust
   println!("File size: {} bytes", file_data.len());
   ```
4. Verify MIME type:
   ```rust
   let mime_type = get_mime_type(&attachment.extension);
   println!("MIME type: {}", mime_type);
   ```

**Solution:**
```rust
// Use failover with multiple servers
let servers = vec![
    "https://blossom.primal.net".to_string(),
    "https://blossom2.example.com".to_string(),
];

let url = upload_blob_with_failover(
    keys,
    servers,
    file_data,
    Some("image/png"),
).await?;
```

### Slow Uploads

**Issue:** Uploads taking too long

**Causes:**
- Large files
- Slow network connection
- Server load

**Solutions:**
1. Compress files before upload
2. Increase chunk size:
   ```rust
   let params = UploadParams {
       chunk_size: 128 * 1024, // 128 KB
       ..Default::default()
   };
   ```
3. Use multiple servers for failover
4. Monitor progress:
   ```rust
   let progress_callback = std::sync::Arc::new(move |percentage, bytes| {
       if let Some(pct) = percentage {
           println!("Upload progress: {}%", pct);
       }
       Ok(())
   });
   ```

## Group Messaging Problems

### Group Join Failures

**Error:** `Failed to process welcome` or `No welcomes available`

**Causes:**
- Invalid welcome event
- Group already joined
- MLS engine not initialized

**Debugging Steps:**
1. Verify welcome event is valid:
   ```rust
   let welcome_event = UnsignedEvent::from_json(json_str)?;
   println!("Welcome event: {:?}", welcome_event);
   ```
2. Check if group already exists:
   ```rust
   let groups = engine.get_groups()?;
   println!("Existing groups: {:?}", groups);
   ```
3. Ensure MLS engine is initialized:
   ```rust
   let device_mdk = MlsGroup::new_persistent()?;
   ```

**Solution:**
```rust
// Process welcome event
let welcome_event = UnsignedEvent::from_json(welcome_json)?;
let group = bot.quick_join_group(welcome_event).await?;

// Verify group was created
let group_info = bot.get_group(group.group.mls_group_id).await?;
println!("Group created: {:?}", group_info);
```

### Message Not Received in Group

**Issue:** Messages sent but not received by group members

**Causes:**
- Member not properly added to group
- Key package not published
- Relay not receiving events

**Debugging Steps:**
1. Verify key package was published:
   ```rust
   let engine = device_mdk.engine()?;
   let key_packages = engine.get_key_packages()?;
   println!("Key packages: {:?}", key_packages);
   ```
2. Check relay connections:
   ```rust
   println!("Connected relays: {:?}", bot.client.relays());
   ```
3. Verify group membership:
   ```rust
   let members = engine.get_group_members(&group_id)?;
   println!("Group members: {:?}", members);
   ```

**Solution:**
```rust
// Ensure key package is published
let engine = device_mdk.engine()?;
engine.create_key_package_for_event(&keys.public_key(), [relay_url])?;

// Send message with verification
let result = group.send_group_message("test").await;
println!("Message sent: {:?}", result);
```

## Logging and Debugging

### Enabling Debug Logs

**Basic Logging:**
```bash
RUST_LOG=debug cargo run
```

**Verbose Logging:**
```bash
RUST_LOG=trace cargo run
```

**Filter Specific Modules:**
```bash
RUST_LOG=vector_sdk=debug,nostr_sdk=info cargo run
```

### Debugging Event Processing

**Inspect Incoming Events:**
```rust
use log::debug;
use nostr_sdk::prelude::*;

while let Some(event) = bot.client.next_incoming_message().await {
    match event {
        RelayPoolNotification::Message { message, .. } => {
            debug!("Received event: {:?}", message.event);
            debug!("Event kind: {:?}", message.event.kind);
            debug!("Event tags: {:?}", message.event.tags);
        }
        _ => {}
    }
}
```

### Monitoring Upload Progress

**Track Upload Progress:**
```rust
let progress_callback = std::sync::Arc::new(move |percentage, bytes| {
    if let Some(pct) = percentage {
        println!("Upload progress: {}%", pct);
    }
    if let Some(b) = bytes {
        println!("Bytes sent: {}", b);
    }
    Ok(())
});

let url = upload_data_with_progress(
    &keys,
    &server_config,
    file_data,
    Some("image/png"),
    None,
    progress_callback,
    None,
    None,
).await?;
```

## Performance Issues

### Slow Message Processing

**Issue:** Messages take too long to process

**Causes:**
- Too many subscriptions
- Large number of events
- Inefficient event handling

**Solutions:**
1. Limit event count in filters:
   ```rust
   let filter = Filter::new()
       .pubkey(recipient)
       .kind(Kind::GiftWrap)
       .limit(50); // Limit to 50 events
   ```
2. Use efficient event processing:
   ```rust
   // Process events in batches
   let events = bot.client.fetch_inbox().await?;
   for event in events {
       // Process each event
   }
   ```
3. Optimize subscriptions:
   ```rust
   // Unsubscribe when not needed
   bot.client.unsubscribe(subscription_id).await?;
   ```

### High Memory Usage

**Issue:** Application using too much memory

**Causes:**
- Caching too many events
- Large file buffers
- Memory leaks in dependencies

**Solutions:**
1. Limit event cache size:
   ```rust
   let mut client = Client::builder()
       .signer(keys)
       .max_memory_events(1000) // Limit cached events
       .build();
   ```
2. Clear file buffers after use:
   ```rust
   let attachment = load_file("file.png")?;
   // Use attachment
   drop(attachment); // Explicitly drop
   ```
3. Use streaming for large files:
   ```rust
   let params = UploadParams {
       chunk_size: 64 * 1024, // 64 KB chunks
       ..Default::default()
   };
   ```

## FAQ

### Q: How do I check if a relay is working?

**A:** Use the Nostr network monitor:
- https://nostr.watch
- https://nostr.band

Or test with curl:
```bash
curl -v https://jskitty.cat/nostr
```

### Q: Why are my messages not encrypted?

**A:** Ensure you're using the correct methods:
- For direct messages: `send_private_message()`
- For groups: `send_group_message()`
- Verify encryption parameters are being used

### Q: How do I handle decryption errors gracefully?

**A:** Wrap decryption in error handling:
```rust
match crypto::decrypt_data(&encrypted_data, &params) {
    Ok(decrypted) => {
        // Process decrypted data
    }
    Err(e) => {
        // Log error and continue
        error!("Decryption failed: {}", e);
        return Err(VectorBotError::Crypto(e));
    }
}
```

### Q: Why is the bot not receiving messages?

**A:** Check these common issues:
1. Subscription not set up
2. Wrong public key in filter
3. Relays not supporting required NIPs
4. Bot not connected to relays

**Debugging:**
```rust
// Check subscriptions
let subs = bot.client.subscriptions();
println!("Active subscriptions: {:?}", subs);

// Check relay connections
let relays = bot.client.relays();
println!("Connected relays: {:?}", relays);
```

### Q: How do I increase upload timeout?

**A:** Configure upload timeout:
```rust
let config = UploadConfig {
    connect_timeout: std::time::Duration::from_secs(30),
    ..Default::default()
};

let url = upload_data_with_progress(
    &keys,
    &server_config,
    file_data,
    Some("image/png"),
    None,
    progress_callback,
    None,
    Some(config),
).await?;
```

### Q: Why are file uploads failing?

**A:** Check these common issues:
1. Network connectivity
2. Server availability
3. File size limits
4. Invalid MIME type
5. Authentication failures

**Debugging:**
```rust
// Test with a small file first
let small_file = vec![0u8; 1024]; // 1 KB test file
let url = upload_blob(&keys, &server_url, small_file, Some("application/octet-stream")).await?;

// Then try with actual file
let file_data = std::fs::read("large_file.png")?;
let url = upload_blob(&keys, &server_url, file_data, Some("image/png")).await?;
```

### Q: How do I debug MLS group issues?

**A:** Enable detailed logging:
```bash
RUST_LOG=vector_sdk::mls=trace,mdk=trace cargo run
```

Check group state:
```rust
let engine = device_mdk.engine()?;
let groups = engine.get_groups()?;
println!("Groups: {:?}", groups);

let members = engine.get_group_members(&group_id)?;
println!("Members: {:?}", members);

let messages = engine.get_messages(&group_id)?;
println!("Messages: {:?}", messages);
```

### Q: Why is the bot slow to start?

**A:** Common causes:
1. Too many relays connecting
2. Large number of subscriptions
3. Slow network connection
4. MLS engine initialization

**Solutions:**
1. Reduce number of relays
2. Limit initial subscriptions
3. Add connection timeouts
4. Pre-initialize MLS engine

## Resources

- [Nostr Protocol Specifications](https://github.com/nostr-protocol/nips)
- [NIP-59: Gift Wrap](https://github.com/nostr-protocol/nips/blob/master/59.md)
- [NIP-40: Application-Specific Data](https://github.com/nostr-protocol/nips/blob/master/40.md)
- [NIP-25: Reactions](https://github.com/nostr-protocol/nips/blob/master/25.md)
- [Rust Logging](https://docs.rs/log/latest/log/)
- [Tokio Async Runtime](https://tokio.rs/)
- [Nostr Relay List](https://nostr.watch)
