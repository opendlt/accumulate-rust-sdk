//! Cryptographic utilities for Accumulate protocol
//!
//! Provides Ed25519 key generation, signing, and verification utilities
//! that match the TypeScript SDK implementation

use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

/// Ed25519 key utilities
pub struct Ed25519Helper;

impl Ed25519Helper {
    /// Generate a new Ed25519 keypair using secure random
    pub fn generate_keypair() -> Keypair {
        let mut csprng = OsRng {};
        Keypair::generate(&mut csprng)
    }

    /// Create keypair from a 32-byte seed (deterministic)
    pub fn keypair_from_seed(seed: &[u8; 32]) -> Result<Keypair, CryptoError> {
        Keypair::from_bytes(seed).map_err(|e| CryptoError::InvalidSeed(e.to_string()))
    }

    /// Create keypair from a hex-encoded private key
    pub fn keypair_from_hex(hex_key: &str) -> Result<Keypair, CryptoError> {
        let bytes = hex::decode(hex_key).map_err(|e| CryptoError::InvalidHex(e.to_string()))?;
        if bytes.len() != 64 {
            return Err(CryptoError::InvalidKeyLength(bytes.len()));
        }

        let mut seed_bytes = [0u8; 32];
        seed_bytes.copy_from_slice(&bytes[0..32]);

        Self::keypair_from_seed(&seed_bytes)
    }

    /// Extract public key from keypair as bytes
    pub fn public_key_bytes(keypair: &Keypair) -> [u8; 32] {
        keypair.public.to_bytes()
    }

    /// Extract private key from keypair as bytes (seed + public key)
    pub fn private_key_bytes(keypair: &Keypair) -> [u8; 64] {
        keypair.to_bytes()
    }

    /// Sign a message hash with Ed25519
    pub fn sign(keypair: &Keypair, message_hash: &[u8]) -> Signature {
        keypair.sign(message_hash)
    }

    /// Verify an Ed25519 signature
    pub fn verify(
        public_key: &PublicKey,
        message_hash: &[u8],
        signature: &Signature,
    ) -> Result<(), CryptoError> {
        public_key
            .verify(message_hash, signature)
            .map_err(|e| CryptoError::VerificationFailed(e.to_string()))
    }

    /// Create public key from bytes
    pub fn public_key_from_bytes(bytes: &[u8; 32]) -> Result<PublicKey, CryptoError> {
        PublicKey::from_bytes(bytes).map_err(|e| CryptoError::InvalidPublicKey(e.to_string()))
    }

    /// Create signature from bytes
    pub fn signature_from_bytes(bytes: &[u8; 64]) -> Result<Signature, CryptoError> {
        Signature::from_bytes(bytes).map_err(|e| CryptoError::InvalidSignature(e.to_string()))
    }

    /// Sign a JSON value using canonical JSON encoding
    pub fn sign_json(keypair: &Keypair, json_value: &serde_json::Value) -> Signature {
        let canonical = super::canonical_json(json_value);
        let hash = sha256_bytes(canonical.as_bytes());
        Self::sign(keypair, &hash)
    }

    /// Verify a signature against a JSON value using canonical JSON encoding
    pub fn verify_json(
        public_key: &PublicKey,
        json_value: &serde_json::Value,
        signature: &Signature,
    ) -> Result<(), CryptoError> {
        let canonical = super::canonical_json(json_value);
        let hash = sha256_bytes(canonical.as_bytes());
        Self::verify(public_key, &hash, signature)
    }
}

/// Hash utilities
pub struct HashHelper;

impl HashHelper {
    /// Compute SHA-256 hash of data
    pub fn sha256(data: &[u8]) -> [u8; 32] {
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }

    /// Compute SHA-256 hash and return as hex string
    pub fn sha256_hex(data: &[u8]) -> String {
        hex::encode(Self::sha256(data))
    }

    /// Compute SHA-256 hash of canonical JSON
    pub fn sha256_json(value: &serde_json::Value) -> [u8; 32] {
        let canonical = super::canonical_json(value);
        Self::sha256(canonical.as_bytes())
    }

    /// Compute SHA-256 hash of canonical JSON as hex string
    pub fn sha256_json_hex(value: &serde_json::Value) -> String {
        hex::encode(Self::sha256_json(value))
    }
}

/// Re-export for convenience
pub use super::{canonical_json, sha256_bytes, sha256_hex};

/// Cryptographic errors
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("Invalid seed: {0}")]
    InvalidSeed(String),

    #[error("Invalid hex encoding: {0}")]
    InvalidHex(String),

    #[error("Invalid key length: expected 64 bytes, got {0}")]
    InvalidKeyLength(usize),

    #[error("Invalid public key: {0}")]
    InvalidPublicKey(String),

    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    #[error("Signature verification failed: {0}")]
    VerificationFailed(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_keypair_generation() {
        let keypair = Ed25519Helper::generate_keypair();
        let pub_bytes = Ed25519Helper::public_key_bytes(&keypair);
        assert_eq!(pub_bytes.len(), 32);

        let priv_bytes = Ed25519Helper::private_key_bytes(&keypair);
        assert_eq!(priv_bytes.len(), 64);
    }

    #[test]
    fn test_deterministic_keypair() {
        let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let keypair1 = Ed25519Helper::keypair_from_hex(hex_key).unwrap();
        let keypair2 = Ed25519Helper::keypair_from_hex(hex_key).unwrap();

        let pub1 = Ed25519Helper::public_key_bytes(&keypair1);
        let pub2 = Ed25519Helper::public_key_bytes(&keypair2);

        assert_eq!(pub1, pub2);

        // Test against expected public key from fixtures
        let expected_pub = "3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29";
        assert_eq!(hex::encode(pub1), expected_pub);
    }

    #[test]
    fn test_signing_and_verification() {
        let keypair = Ed25519Helper::generate_keypair();
        let message = b"Hello, Accumulate!";
        let hash = HashHelper::sha256(message);

        let signature = Ed25519Helper::sign(&keypair, &hash);
        let result = Ed25519Helper::verify(&keypair.public, &hash, &signature);

        assert!(result.is_ok());
    }

    #[test]
    fn test_json_signing() {
        let hex_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
        let keypair = Ed25519Helper::keypair_from_hex(hex_key).unwrap();

        let json_value = json!({
            "header": {
                "principal": "acc://alice.acme/tokens",
                "timestamp": 1234567890123u64
            },
            "body": {
                "type": "send-tokens",
                "to": [{
                    "url": "acc://bob.acme/tokens",
                    "amount": "1000"
                }]
            }
        });

        let signature = Ed25519Helper::sign_json(&keypair, &json_value);
        let result = Ed25519Helper::verify_json(&keypair.public, &json_value, &signature);

        assert!(result.is_ok());
    }

    #[test]
    fn test_hash_consistency() {
        let data = b"test data";
        let hash1 = HashHelper::sha256(data);
        let hash2 = HashHelper::sha256(data);

        assert_eq!(hash1, hash2);

        let hex1 = HashHelper::sha256_hex(data);
        let hex2 = hex::encode(hash1);

        assert_eq!(hex1, hex2);
    }

    #[test]
    fn test_json_hash_matches_fixture() {
        let json_value = json!({
            "header": {
                "principal": "acc://alice.acme/tokens",
                "timestamp": 1234567890123u64
            },
            "body": {
                "type": "send-tokens",
                "to": [{
                    "url": "acc://bob.acme/tokens",
                    "amount": "1000"
                }]
            }
        });

        let hash_hex = HashHelper::sha256_json_hex(&json_value);
        let expected = "4be49c59c717f1984646998cecac0e5225378d9bbe2e18928272a85b7dfcb608";

        assert_eq!(hash_hex, expected);
    }

    #[test]
    fn test_invalid_key_handling() {
        let result = Ed25519Helper::keypair_from_hex("invalid_hex");
        assert!(result.is_err());

        let result = Ed25519Helper::keypair_from_hex("123"); // Too short
        assert!(result.is_err());
    }
}
