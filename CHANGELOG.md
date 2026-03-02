# Changelog

All notable changes to the Vector SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2026-01-20

### Added
- Initial implementation of MLS (Message Layer Security) support for group messaging
- Blossom media server integration for file uploads with automatic failover
- Progress tracking for file uploads
- Typing indicators for both direct messages and group messages
- Reaction support for messages (NIP-25)
- Image metadata extraction (blurhash, dimensions)
- File type inference from bytes for attachments without extensions
- Comprehensive error handling with `VectorBotError` enum
- MLS Implementation Status documentation in README.md
- Enhanced security documentation for key management and MLS

### Changed
- Improved error handling - replaced `panic!()` calls with proper error propagation
- Enhanced logging throughout the library
- Better organization of modules and exports
- Version bump from 0.2.1 to 0.3.0 to reflect MLS additions

### Fixed
- Various bug fixes and stability improvements
- Proper error handling in bot initialization
- Type conversion issues in MLS group operations

### Security
- Strong encryption using AES-256-GCM
- Secure random key generation for encryption
- Proper handling of cryptographic operations
- Updated security documentation with key management guidelines
- MLS storage security considerations

### Deprecated
- None

### Removed
- None

### MLS Implementation Status

The following MLS features are fully implemented and ready for use:
- ✅ Group joining via welcome events
- ✅ Group message sending and processing
- ✅ Group typing indicators
- ✅ Group file attachments
- ✅ Group reactions
- ✅ Persistent SQLite-backed storage

The following MLS functions exist as placeholders and will be implemented in future versions:
- ⚠️ `create_group()` - Group creation
- ⚠️ `add_member_device()` - Adding members
- ⚠️ `leave_group()` - Leaving groups
- ⚠️ `remove_member_device_from_group()` - Removing members

These placeholder functions return errors if called but are included to provide a complete API surface for future expansion.

## [0.2.1] - 2024-01-15

### Added
- Initial public release of Vector SDK
- Core functionality for creating and managing vector bots
- Support for sending private messages using Nostr gift wrap (NIP-59)
- File attachment support with encryption and upload to media servers
- Metadata management for bot profiles
- Subscription handling for gift wrap events
- Basic client configuration with relay management

### Features
- `VectorBot` struct for bot management
- `Channel` struct for direct messaging
- `Client` module for Nostr client configuration
- `Metadata` module for profile management
- `Subscription` module for event subscriptions
- `Crypto` module for encryption/decryption
- `Upload` module for file uploads with progress tracking

### Dependencies
- `nostr_sdk`: Core Nostr protocol implementation
- `tokio`: Async runtime
- `aes` and `aes_gcm`: AES-256-GCM encryption
- `reqwest`: HTTP client for file uploads
- `sha2`: SHA-256 hashing
- `url`: URL parsing and manipulation
- `log`: Logging support
- `thiserror`: Error handling
- `mime_guess`: MIME type detection
- `magical_rs`: File type detection from bytes

## [0.1.0] - 2023-12-01

### Added
- Initial development version
- Basic bot structure
- Core module organization
- Initial documentation

[0.3.0]: https://github.com/VectorPrivacy/Vector-SDK/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/VectorPrivacy/Vector-SDK/compare/v0.1.0...v0.2.1
[0.1.0]: https://github.com/VectorPrivacy/Vector-SDK/releases/tag/v0.1.0
