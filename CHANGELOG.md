# Changelog

All notable changes to the Accumulate Rust SDK will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Complete Rust SDK implementation with V2/V3 unified support
- DevNet discovery binary for automatic configuration
- Comprehensive examples demonstrating zero-to-hero workflows
- TypeScript SDK parity with byte-for-byte compatibility
- Canonical JSON encoding matching TS implementation
- Ed25519 cryptographic utilities for signing and verification
- Transaction envelope creation and verification
- Integration tests for DevNet compatibility
- Conformance tests against TypeScript SDK fixtures
- GitHub Actions CI/CD with multi-platform testing
- Code coverage reporting with cargo-llvm-cov
- Production-ready linting and formatting configuration

### Changed
- N/A (initial release)

### Deprecated
- N/A (initial release)

### Removed
- N/A (initial release)

### Fixed
- N/A (initial release)

### Security
- N/A (initial release)

## [0.1.0] - TBD

### Added
- Initial release of Accumulate Rust SDK
- Core client functionality for V2 and V3 APIs
- DevNet-first development experience
- Complete documentation and examples
- Production-ready CI/CD pipeline

---

## Guidelines for Maintainers

When releasing a new version:

1. Update the version number in `Cargo.toml`
2. Move items from `[Unreleased]` to a new version section
3. Add a new empty `[Unreleased]` section
4. Include the release date in ISO format (YYYY-MM-DD)
5. Create a git tag with the version number (e.g., `v0.1.0`)

### Categories

- **Added** for new features
- **Changed** for changes in existing functionality
- **Deprecated** for soon-to-be removed features
- **Removed** for now removed features
- **Fixed** for any bug fixes
- **Security** in case of vulnerabilities