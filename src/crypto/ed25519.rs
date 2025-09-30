use ed25519_dalek::{Keypair, PublicKey, SecretKey, Signature, Signer, Verifier};
use rand::rngs::OsRng;
use sha2::{Digest, Sha256};

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

    pub fn from_seed(seed: &[u8; 32]) -> Result<Self, ed25519_dalek::SignatureError> {
        let secret_key = SecretKey::from_bytes(seed)?;
        let public_key = PublicKey::from(&secret_key);
        let keypair = Keypair {
            secret: secret_key,
            public: public_key,
        };
        Ok(Self::new(keypair))
    }

    pub fn sign(&self, message: &[u8]) -> Signature {
        self.keypair.sign(message)
    }

    pub fn public_key(&self) -> &PublicKey {
        &self.keypair.public
    }

    pub fn public_key_bytes(&self) -> [u8; 32] {
        self.keypair.public.to_bytes()
    }

    /// Create signer from 64-byte combined key (private + public)
    pub fn from_keypair_bytes(bytes: &[u8; 64]) -> Result<Self, ed25519_dalek::SignatureError> {
        let keypair = Keypair::from_bytes(bytes)?;
        Ok(Self::new(keypair))
    }

    /// Get private key as 32-byte array
    pub fn private_key_bytes(&self) -> [u8; 32] {
        self.keypair.secret.to_bytes()
    }

    /// Get full keypair as 64-byte array (private + public)
    pub fn keypair_bytes(&self) -> [u8; 64] {
        self.keypair.to_bytes()
    }

    /// Sign a pre-hashed message (compatible with TS SDK)
    pub fn sign_prehashed(&self, hash: &[u8; 32]) -> Signature {
        self.keypair.sign(hash)
    }
}

pub fn verify_signature(
    public_key: &[u8; 32],
    message: &[u8],
    signature: &[u8; 64],
) -> Result<(), ed25519_dalek::SignatureError> {
    let public_key = PublicKey::from_bytes(public_key)?;
    let signature = Signature::from_bytes(signature)?;
    public_key.verify(message, &signature)
}

/// Verify Ed25519 signature against pre-hashed message
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
        let signature_bytes = signature.to_bytes();

        assert!(verify_signature(&public_key_bytes, message, &signature_bytes).is_ok());
    }

    #[test]
    fn test_from_seed() {
        let seed = [42u8; 32];
        let signer = Ed25519Signer::from_seed(&seed).unwrap();
        let message = b"Deterministic test";
        let signature = signer.sign(message);

        let signer2 = Ed25519Signer::from_seed(&seed).unwrap();
        let signature2 = signer2.sign(message);

        assert_eq!(signature.to_bytes(), signature2.to_bytes());
        assert_eq!(signer.public_key_bytes(), signer2.public_key_bytes());
    }
}
