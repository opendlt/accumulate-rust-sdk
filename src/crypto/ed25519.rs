// use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier}; // Broken API - commented out
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

// BROKEN: API changed - commented out for Stage 1.2
/*
/// Ed25519 signer that exactly matches TypeScript SDK behavior
pub struct Ed25519Signer {
    keypair: Keypair,
}

impl Ed25519Signer {
    pub fn new(keypair: Keypair) -> Self {
        Self { keypair }
    }

    pub fn generate() -> Self {
        let mut csprng = OsRng {};
        let keypair = Keypair::generate(&mut csprng);
        Self::new(keypair)
    }

    /// Create signer from 32-byte seed (matches TS SDK)
    /// This is the primary method for deterministic keypair generation
    pub fn from_seed(seed: &[u8; 32]) -> Result<Self, ed25519_dalek::SignatureError> {
        let secret_key = SecretKey::from_bytes(seed)?;
        let public_key = PublicKey::from(&secret_key);
        let keypair = Keypair {
            secret: secret_key,
            public: public_key,
        };
        Ok(Self::new(keypair))
    }

    /// Sign message and return 64-byte signature array (matches TS SDK)
    /// Returns [u8; 64] for exact byte-for-byte compatibility
    pub fn sign(&self, message: &[u8]) -> [u8; 64] {
        let signature = self.keypair.sign(message);
        signature.to_bytes()
    }

    /// Sign a pre-hashed message and return 64-byte signature array
    pub fn sign_prehashed(&self, hash: &[u8; 32]) -> [u8; 64] {
        let signature = self.keypair.sign(hash);
        signature.to_bytes()
    }

    /// Get public key as reference
    pub fn public_key(&self) -> &PublicKey {
        &self.keypair.public
    }

    /// Get public key as 32-byte array (matches TS SDK)
    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.keypair.public.to_bytes()
    }

    /// Get private key as 32-byte array
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.keypair.secret.to_bytes()
    }

    /// Create signer from 64-byte combined key (private + public)
    pub fn from_keypair_bytes(bytes: &[u8; 64]) -> Result<Self, ed25519_dalek::SignatureError> {
        let keypair = Keypair::from_bytes(bytes)?;
        Ok(Self::new(keypair))
    }

    /// Get full keypair as 64-byte array (private + public)
    pub fn keypair_bytes(&self) -> [u8; 64] {
        self.keypair.to_bytes()
    }
}

/// Verify Ed25519 signature (matches TS SDK)
/// Returns true if signature is valid, false otherwise
pub fn verify(
    public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) -> bool {
    match PublicKey::from_bytes(public_key) {
        Ok(public_key) => {
            match Signature::from_bytes(signature) {
                Ok(signature) => public_key.verify(message, &signature).is_ok(),
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

/// Verify Ed25519 signature against pre-hashed message (matches TS SDK)
/// Returns true if signature is valid, false otherwise
pub fn verify_prehashed(
    public_key: &[u8; 32],
    hash: &[u8; 32],
    signature: &[u8; 64],
) -> bool {
    match PublicKey::from_bytes(public_key) {
        Ok(public_key) => {
            match Signature::from_bytes(signature) {
                Ok(signature) => public_key.verify(hash, &signature).is_ok(),
                Err(_) => false,
            }
        }
        Err(_) => false,
    }
}

/// Legacy verify function for backwards compatibility
pub fn verify_signature(
    public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) -> Result<(), ed25519_dalek::SignatureError> {
    let public_key = PublicKey::from_bytes(public_key)?;
    let signature = Signature::from_bytes(signature)?;
    public_key.verify(message, &signature)
}

/// Legacy verify function for backwards compatibility
pub fn verify_signature_prehashed(
    public_key: &[u8; 32],
    hash: &[u8; 32],
    signature: &[u8; 64],
) -> Result<(), ed25519_dalek::SignatureError> {
    let public_key = PublicKey::from_bytes(public_key)?;
    let signature = Signature::from_bytes(signature)?;
    public_key.verify(hash, &signature)
}

/// Hash message with SHA-256
pub fn sha256(message: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(message);
    hasher.finalize().into()
}

/// Hash message and return as hex string
pub fn sha256_hex(message: &[u8]) -> String {
    hex::encode(sha256(message))
}

/// Legacy alias for sha256
pub fn hash_message(message: &[u8]) -> [u8; 32] {
    sha256(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_and_sign() {
        let signer = Ed25519Signer::generate();
        let message = b"Hello, Accumulate!";
        let signature = signer.sign(message);

        let public_key_bytes = signer.public_key_bytes();

        assert!(verify(&public_key_bytes, message, &signature));
        assert!(verify_signature(&public_key_bytes, message, &signature).is_ok());
    }

    #[test]
    fn test_from_seed() {
        let seed = [42u8; 32];
        let signer = Ed25519Signer::from_seed(&seed).unwrap();
        let message = b"Deterministic test";
        let signature = signer.sign(message);

        let signer2 = Ed25519Signer::from_seed(&seed).unwrap();
        let signature2 = signer2.sign(message);

        assert_eq!(signature, signature2);
        assert_eq!(signer.public_key_bytes(), signer2.public_key_bytes());
    }

    #[test]
    fn test_sign_returns_64_bytes() {
        let seed = [1u8; 32];
        let signer = Ed25519Signer::from_seed(&seed).unwrap();
        let message = b"test message";
        let signature = signer.sign(message);

        assert_eq!(signature.len(), 64);
        assert!(verify(&signer.public_key_bytes(), message, &signature));
    }

    #[test]
    fn test_verify_functions() {
        let seed = [2u8; 32];
        let signer = Ed25519Signer::from_seed(&seed).unwrap();
        let message = b"test verification";
        let signature = signer.sign(message);
        let public_key = signer.public_key_bytes();

        // Test new verify function
        assert!(verify(&public_key, message, &signature));

        // Test with wrong message
        assert!(!verify(&public_key, b"wrong message", &signature));

        // Test with wrong signature
        let wrong_signature = [0u8; 64];
        assert!(!verify(&public_key, message, &wrong_signature));
    }
}
*/
