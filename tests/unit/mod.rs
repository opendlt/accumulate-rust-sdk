//! Unit tests organized by functionality
//!
//! This module contains unit tests categorized by the module they test:
//! - codec: Encoding, decoding, canonicalization, binary formats
//! - crypto: Cryptographic operations, signatures, key management
//! - protocol: Protocol types, enums, transactions, type validation
//! - runtime: API, RPC, core runtime functionality, error handling

pub mod codec;
pub mod crypto;
pub mod protocol;
pub mod runtime;