# Security Documentation

This document provides an overview of the security features, best practices, and considerations for the Vector SDK.

## Table of Contents

- [Overview](#overview)
- [Cryptography](#cryptography)
- [Data Protection](#data-protection)
- [Threat Model](#threat-model)
- [Best Practices](#best-practices)
- [Vulnerability Reporting](#vulnerability-reporting)
- [Dependencies](#dependencies)

## Overview

Vector SDK is designed with security as a primary concern. It provides end-to-end encryption for all communications and implements industry-standard cryptographic protocols to protect user privacy.

## Cryptography

### Encryption Algorithms

#### AES-256-GCM
- **Purpose**: Encrypting file attachments and sensitive data
- **Key Size**: 256-bit (32 bytes)
- **Nonce Size**: 128-bit (16 bytes)
- **Authentication**: Galois/Counter Mode (GCM) provides authenticated encryption
- **Implementation**: `aes-gcm` crate with `Aes256` cipher

#### SHA-256
- **Purpose**: Hashing files for integrity verification
- **Output**: 256-bit (32 byte) hash
- **Implementation**: `sha2` crate

### Key Management

- **Key Generation**: Cryptographically secure random keys using `rand::thread_rng()`
- **Key Storage**: Keys are stored in memory and not persisted to disk
- **Key Rotation**: Applications should implement their own key rotation policies

### Message Layer Security (MLS)

Vector SDK integrates with the [nostr-mls](https://github.com/nostr-protocol/nips/tree/master/nips) protocol for group messaging:

- **End-to-End Encryption**: All group messages are encrypted
- **Forward Secrecy**: Ephemeral keys provide forward secrecy
- **Key Package Management**: Automatic publishing of key packages to relays
- **Group Membership**: Secure group creation, joining, and member management

## Data Protection

### Private Messaging

- **Protocol**: NIP-59 (Gift Wrap) for direct messages
- **Encryption**: Each message is encrypted with a unique key
- **Recipient Verification**: Messages are wrapped for specific recipients

### File Attachments

1. **Encryption**: Files are encrypted with AES-256-GCM before upload
2. **Upload**: Encrypted files are uploaded to Blossom media servers
3. **Metadata**: Encryption parameters (key, nonce) are sent separately
4. **Integrity**: SHA-256 hash of original file is included in metadata

### Typing Indicators

- **Protocol**: NIP-40 (Application-Specific Data)
- **Expiration**: Typing indicators expire after 30 seconds
- **Encryption**: Typing indicators are encrypted like regular messages

### Reactions

- **Protocol**: NIP-25 (Reactions)
- **Encryption**: Reactions are encrypted and wrapped for recipients
- **Content**: Only emoji content is sent (no additional metadata)

## Threat Model

### Threats Addressed

| Threat | Mitigation |
|--------|------------|
| Eavesdropping | End-to-end encryption with AES-256-GCM |
| Message Tampering | GCM authentication tags verify integrity |
| Impersonation | Nostr public keys authenticate senders |
| Man-in-the-Middle | TLS for relay connections, encrypted content |
| Replay Attacks | Timestamps and sequence numbers prevent replay |
| File Interception | Files encrypted before upload, keys sent separately |

### Threats Not Addressed

- **Malicious Relays**: Relays can withhold or delay messages (standard Nostr limitation)
- **Metadata Leakage**: Profile information (name, picture) is public
- **Key Compromise**: If private keys are compromised, past messages can be decrypted
- **Client-Side Vulnerabilities**: Applications using the SDK must implement secure practices

## Best Practices

### For Application Developers

1. **Key Management**:
   - Store private keys securely (use platform keychains)
   - Never hardcode or commit private keys to version control
   - Implement proper key backup and recovery

2. **Error Handling**:
   - Never expose cryptographic errors to end users
   - Log errors securely (no sensitive data in logs)
   - Handle decryption failures gracefully

3. **Network Security**:
   - Use secure relay connections (wss://)
   - Validate relay URLs before connecting
   - Implement connection timeouts

4. **Data Handling**:
   - Clear sensitive data from memory when no longer needed
   - Validate all file inputs before processing
   - Limit file sizes to prevent DoS attacks

5. **Logging**:
   - Avoid logging encrypted content
   - Mask sensitive information in logs
   - Use appropriate log levels

### For End Users

1. **Key Security**:
   - Protect your private keys
   - Use strong passphrases for key encryption
   - Backup your keys securely

2. **Relay Selection**:
   - Use trusted relays
   - Diversify relay connections for redundancy

3. **File Sharing**:
   - Verify file sources before opening
   - Check file hashes when available
   - Be cautious with executable files

## Vulnerability Reporting

If you discover a security vulnerability in Vector SDK, please follow these steps:

1. **Do not** open a public issue on GitHub
2. **Do not** discuss the vulnerability in public channels
3. **Email** the security team at: `security@vectorprivacy.com`
4. **Include** as much detail as possible:
   - Steps to reproduce
   - Impact assessment
   - Potential mitigations
   - Your contact information

The security team will:
- Acknowledge receipt within 48 hours
- Provide updates on the investigation
- Work on a fix and coordinate disclosure
- Credit responsible disclosers in release notes

## Dependencies

Vector SDK uses the following security-critical dependencies:

| Dependency | Purpose | Version | Notes |
|------------|---------|---------|-------|
| `nostr_sdk` | Nostr protocol implementation | Latest | Includes NIP-59 (Gift Wrap) |
| `aes-gcm` | AES-256-GCM encryption | Latest | FIPS 197 compliant |
| `sha2` | SHA-256 hashing | Latest | FIPS 180-4 compliant |
| `rand` | Cryptographic RNG | Latest | Uses OS-provided CSPRNG |
| `reqwest` | HTTP client | Latest | Used for file uploads |
| `mdk` | Message Layer Security | Latest | For group messaging |

### Dependency Security

- All dependencies are kept up-to-date
- Security advisories are monitored
- Vulnerable dependencies are patched promptly

## Security Checklist for Applications

When building applications with Vector SDK, consider this checklist:

- [ ] Private keys are stored securely
- [ ] Sensitive data is not logged
- [ ] Network connections use TLS
- [ ] File uploads have size limits
- [ ] Error messages don't expose system details
- [ ] Cryptographic operations are properly handled
- [ ] User inputs are validated
- [ ] Session management is secure
- [ ] Dependencies are kept updated
- [ ] Security headers are set (for web apps)

## Resources

- [NIP-59: Gift Wrap](https://github.com/nostr-protocol/nips/blob/master/59.md)
- [NIP-40: Application-Specific Data](https://github.com/nostr-protocol/nips/blob/master/40.md)
- [NIP-25: Reactions](https://github.com/nostr-protocol/nips/blob/master/25.md)
- [AES-GCM Specification](https://nvlpubs.nist.gov/nistpubs/SpecialPublications/NIST.SP.800-38D.pdf)
- [Rust Crypto](https://github.com/RustCrypto)
