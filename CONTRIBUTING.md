# Contributing to Vector SDK

Thank you for considering contributing to Vector SDK! This document provides guidelines for contributing to the project.

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Setup](#development-setup)
- [Coding Standards](#coding-standards)
- [Testing](#testing)
- [Pull Requests](#pull-requests)
- [Documentation](#documentation)
- [Releases](#releases)
- [Security](#security)

## Code of Conduct

Please note that this project is released with a [Contributor Code of Conduct](CODE_OF_CONDUCT.md). By contributing to this project, you agree to abide by its terms.

## Getting Started

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/your-username/Vector-SDK.git
   cd Vector-SDK
   ```
3. **Add the upstream remote** to keep your fork in sync:
   ```bash
   git remote add upstream https://github.com/VectorPrivacy/Vector-SDK.git
   ```

## Development Setup

### Prerequisites

- Rust (latest stable version)
- Cargo (Rust package manager)
- Git
- Optional: Rustfmt and Clippy for code formatting and linting

### Building the Project

```bash
# Clone the repository
git clone https://github.com/VectorPrivacy/Vector-SDK.git
cd Vector-SDK

# Build the project
cargo build

# Build with release optimization
cargo build --release
```

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test -- --nocapture

# Run specific test
cargo test test_name
```

## Coding Standards

### Rust Style Guide

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `snake_case` for variables and functions
- Use `PascalCase` for types and enums
- Use `UPPER_CASE` for constants
- Keep lines under 100 characters when possible
- Use 4 spaces for indentation (Rust's default)

### Documentation

- Document all public items with Rustdoc comments
- Follow the format:
  ```rust
  /// Summary line ending with a period.
  ///
  /// Additional details if needed.
  ///
  /// # Arguments
  ///
  /// * `param` - Description of parameter.
  ///
  /// # Returns
  ///
  /// Description of return value.
  pub fn example_function(param: Type) -> ReturnType {
      // Implementation
  }
  ```

### Error Handling

- Use the `thiserror` crate for defining error types
- Provide clear, actionable error messages
- Implement proper error conversion with `From` trait

### Logging

- Use the `log` crate for logging
- Follow these log levels:
  - `error`: Critical errors that need attention
  - `warn`: Potential issues or deprecated features
  - `info`: Important operational messages
  - `debug`: Detailed debugging information
  - `trace`: Very detailed tracing information

## Testing

### Test Organization

- Unit tests: In the same file as the implementation, in a `#[cfg(test)]` module
- Integration tests: In the `tests/` directory
- Example tests: In the separate examples repository

### Writing Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_name() {
        // Test implementation
        assert_eq!(expected, actual);
    }

    #[test]
    #[should_panic(expected = "error message")]
    fn test_panics() {
        // Code that should panic
    }
}
```

### Test Coverage

- Aim for high test coverage (80%+)
- Test edge cases and error conditions
- Test async code properly with `tokio::test`

## Pull Requests

### Creating a Pull Request

1. **Create a feature branch** from `main`:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following the coding standards

3. **Commit your changes** with clear, descriptive messages:
   ```bash
   git commit -m "feat: add new feature description"
   git commit -m "fix: resolve issue with description"
   ```

4. **Push to your fork**:
   ```bash
   git push origin feature/your-feature-name
   ```

5. **Open a Pull Request** on GitHub with:
   - Clear title describing the change
   - Detailed description of what was changed and why
   - Related issues (if any)
   - Screenshots or examples (if applicable)

### Pull Request Requirements

- All tests must pass
- Code must be properly formatted (run `cargo fmt`)
- Code must pass linting (run `cargo clippy`)
- Documentation must be updated
- Changes must follow the coding standards

## Documentation

### Updating Documentation

- Update the README.md for major changes
- Update CHANGELOG.md for new features and fixes
- Add or update Rustdoc comments for code changes
- Update any relevant documentation files

### Generating Documentation

To generate and view the API documentation:

```bash
# Generate documentation
cargo doc --open

# Generate documentation with nightly features
cargo +nightly doc --open --no-deps
```

## Releases

### Versioning

This project follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html):
- `MAJOR` version when making breaking changes
- `MINOR` version when adding functionality in a backwards-compatible manner
- `PATCH` version when making backwards-compatible bug fixes

### Release Process

1. Update CHANGELOG.md with release notes
2. Update version in Cargo.toml
3. Create a git tag:
   ```bash
   git tag -a vX.Y.Z -m "Release vX.Y.Z"
   git push origin vX.Y.Z
   ```
4. Publish to crates.io:
   ```bash
   cargo publish
   ```

## Security

### Reporting Security Issues

If you discover a security vulnerability, please:
1. Do not open a public issue
2. Email the maintainers directly at security@vectorprivacy.com
3. Include as much detail as possible

### Security Best Practices

- Always use secure random number generation
- Validate all inputs
- Use proper encryption for sensitive data
- Follow the principle of least privilege
- Keep dependencies updated

## Questions

If you have any questions about contributing, please open an issue on GitHub or contact the maintainers.
