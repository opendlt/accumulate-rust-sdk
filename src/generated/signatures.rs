// GENERATED FILE - DO NOT EDIT
// Source: protocol/signatures.yml | Generated: 2025-10-03 20:47:50

use serde::{Serialize, Deserialize};
use hex;

// Helper module for optional hex serialization
mod hex_option {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<[u8; 32]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(bytes) => hex::encode(bytes).serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<[u8; 32]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(hex_str) => {
                let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
                if bytes.len() != 32 {
                    return Err(serde::de::Error::custom("Hash must be 32 bytes"));
                }
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&bytes);
                Ok(Some(hash))
            },
            None => Ok(None),
        }
    }
}

// Helper module for optional bytes hex serialization
mod hex_option_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(bytes) => hex::encode(bytes).serialize(serializer),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(hex_str) => {
                let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
                Ok(Some(bytes))
            },
            None => Ok(None),
        }
    }
}

// Helper module for vector of hashes hex serialization
mod hex_vec_hash {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Vec<[u8; 32]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_strings: Vec<String> = value.iter().map(|hash| hex::encode(hash)).collect();
        hex_strings.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<[u8; 32]>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_strings: Vec<String> = Vec::deserialize(deserializer)?;
        let mut result = Vec::new();
        for hex_str in hex_strings {
            let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
            if bytes.len() != 32 {
                return Err(serde::de::Error::custom("Hash must be 32 bytes"));
            }
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&bytes);
            result.push(hash);
        }
        Ok(result)
    }
}

// Helper module for vector of bytes hex serialization
mod hex_vec_bytes {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S>(value: &Vec<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_strings: Vec<String> = value.iter().map(|bytes| hex::encode(bytes)).collect();
        hex_strings.serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let hex_strings: Vec<String> = Vec::deserialize(deserializer)?;
        let mut result = Vec::new();
        for hex_str in hex_strings {
            let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
            result.push(bytes);
        }
        Ok(result)
    }
}

/// Error type for signature operations
#[derive(Debug, thiserror::Error)]
pub enum SignatureError {
    #[error("Invalid signature format")]
    InvalidFormat,
    #[error("Verification failed: {0}")]
    VerificationFailed(String),
    #[error("Unsupported signature type")]
    UnsupportedType,
}

/// Main signature trait for verification
pub trait AccSignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error>;
    fn sig_type(&self) -> &'static str;
}

/// LegacyED25519Signature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LegacyED25519Signature {
    #[serde(rename = "Timestamp")]
    pub timestamp: u64,
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
}

/// RCD1Signature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RCD1Signature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
}

/// ED25519Signature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ED25519Signature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
}

/// BTCSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BTCSignature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
}

/// BTCLegacySignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BTCLegacySignature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
}

/// ETHSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ETHSignature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
}

/// RsaSha256Signature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RsaSha256Signature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
}

/// EcdsaSha256Signature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EcdsaSha256Signature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
}

/// TypedDataSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TypedDataSignature {
    #[serde(rename = "PublicKey", with = "hex::serde")]
    pub public_key: Vec<u8>,
    #[serde(rename = "Signature", with = "hex::serde")]
    pub signature: Vec<u8>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "SignerVersion")]
    pub signer_version: u64,
    #[serde(rename = "Timestamp")]
    pub timestamp: Option<u64>,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
    #[serde(rename = "Data", with = "hex_option_vec")]
    pub data: Option<Vec<u8>>,
    #[serde(rename = "ChainID")]
    pub chain_i_d: String,
}

/// ReceiptSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReceiptSignature {
    #[serde(rename = "SourceNetwork")]
    pub source_network: String,
    #[serde(rename = "Proof")]
    pub proof: crate::types::MerkleReceipt,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
}

/// PartitionSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PartitionSignature {
    #[serde(rename = "SourceNetwork")]
    pub source_network: String,
    #[serde(rename = "DestinationNetwork")]
    pub destination_network: String,
    #[serde(rename = "SequenceNumber")]
    pub sequence_number: u64,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
}

/// SignatureSet signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureSet {
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "Signer")]
    pub signer: String,
    #[serde(rename = "TransactionHash", with = "hex_option")]
    pub transaction_hash: Option<[u8; 32]>,
    #[serde(rename = "Signatures")]
    pub signatures: Vec<Box<crate::generated::signatures::Signature>>,
    #[serde(rename = "Authority")]
    pub authority: String,
}

/// RemoteSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RemoteSignature {
    #[serde(rename = "Destination")]
    pub destination: String,
    #[serde(rename = "Signature")]
    pub signature: Box<crate::generated::signatures::Signature>,
    #[serde(rename = "Cause", with = "hex_vec_hash")]
    pub cause: Vec<[u8; 32]>,
}

/// DelegatedSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelegatedSignature {
    #[serde(rename = "Signature")]
    pub signature: Box<crate::generated::signatures::Signature>,
    #[serde(rename = "Delegator")]
    pub delegator: String,
}

/// InternalSignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InternalSignature {
    #[serde(rename = "Cause", with = "hex::serde")]
    pub cause: [u8; 32],
    #[serde(rename = "TransactionHash", with = "hex::serde")]
    pub transaction_hash: [u8; 32],
}

/// AuthoritySignature signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthoritySignature {
    #[serde(rename = "Origin")]
    pub origin: String,
    #[serde(rename = "Authority")]
    pub authority: String,
    #[serde(rename = "Vote")]
    pub vote: Option<crate::generated::enums::VoteType>,
    #[serde(rename = "TxID")]
    pub tx_i_d: String,
    #[serde(rename = "Cause")]
    pub cause: String,
    #[serde(rename = "Delegator")]
    pub delegator: Vec<String>,
    #[serde(rename = "Memo")]
    pub memo: Option<String>,
}

/// Main signature dispatch enum
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Signature {
    #[serde(rename = "legacyED25519")]
    LegacyED25519(LegacyED25519Signature),
    #[serde(rename = "rcd1")]
    RCD1(RCD1Signature),
    #[serde(rename = "ed25519")]
    ED25519(ED25519Signature),
    #[serde(rename = "btc")]
    BTC(BTCSignature),
    #[serde(rename = "btcLegacy")]
    BTCLegacy(BTCLegacySignature),
    #[serde(rename = "eth")]
    ETH(ETHSignature),
    #[serde(rename = "rsaSha256")]
    RsaSha256(RsaSha256Signature),
    #[serde(rename = "ecdsaSha256")]
    EcdsaSha256(EcdsaSha256Signature),
    #[serde(rename = "typedData")]
    TypedData(TypedDataSignature),
    #[serde(rename = "receipt")]
    Receipt(ReceiptSignature),
    #[serde(rename = "partition")]
    Partition(PartitionSignature),
    #[serde(rename = "signatureSet")]
    Set(SignatureSet),
    #[serde(rename = "remote")]
    Remote(RemoteSignature),
    #[serde(rename = "delegated")]
    Delegated(DelegatedSignature),
    #[serde(rename = "internal")]
    Internal(InternalSignature),
    #[serde(rename = "authority")]
    Authority(AuthoritySignature),
}

impl Signature {
    pub fn wire_tag(&self) -> &'static str {
        match self {
            Signature::LegacyED25519(_) => "legacyED25519",
            Signature::RCD1(_) => "rcd1",
            Signature::ED25519(_) => "ed25519",
            Signature::BTC(_) => "btc",
            Signature::BTCLegacy(_) => "btcLegacy",
            Signature::ETH(_) => "eth",
            Signature::RsaSha256(_) => "rsaSha256",
            Signature::EcdsaSha256(_) => "ecdsaSha256",
            Signature::TypedData(_) => "typedData",
            Signature::Receipt(_) => "receipt",
            Signature::Partition(_) => "partition",
            Signature::Set(_) => "signatureSet",
            Signature::Remote(_) => "remote",
            Signature::Delegated(_) => "delegated",
            Signature::Internal(_) => "internal",
            Signature::Authority(_) => "authority",
        }
    }
}

impl AccSignature for LegacyED25519Signature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        // Legacy Ed25519 - use same verification as ED25519Signature for now
        use ed25519_dalek::{Signature as Ed25519Sig, VerifyingKey};

        if self.public_key.len() != 32 || self.signature.len() != 64 {
            return Ok(false);
        }

        let mut pub_key_bytes = [0u8; 32];
        pub_key_bytes.copy_from_slice(&self.public_key);

        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(&self.signature);

        match VerifyingKey::from_bytes(&pub_key_bytes) {
            Ok(public_key) => {
                match Ed25519Sig::from_slice(&sig_bytes) {
                    Ok(signature) => {
                        Ok(public_key.verify_strict(message, &signature).is_ok())
                    },
                    Err(_) => Ok(false),
                }
            },
            Err(_) => Ok(false),
        }
    }

    fn sig_type(&self) -> &'static str {
        "legacyED25519"
    }
}

impl AccSignature for RCD1Signature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        // Non-standard signature type - serialization/deserialization only
        // TODO: Stage 1.4 will implement proper verification logic
        Ok(false)
    }

    fn sig_type(&self) -> &'static str {
        "rcd1"
    }
}

impl AccSignature for ED25519Signature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        use ed25519_dalek::{Signature as Ed25519Sig, VerifyingKey};

        if self.public_key.len() != 32 || self.signature.len() != 64 {
            return Ok(false);
        }

        let mut pub_key_bytes = [0u8; 32];
        pub_key_bytes.copy_from_slice(&self.public_key);

        let mut sig_bytes = [0u8; 64];
        sig_bytes.copy_from_slice(&self.signature);

        match VerifyingKey::from_bytes(&pub_key_bytes) {
            Ok(public_key) => {
                match Ed25519Sig::from_slice(&sig_bytes) {
                    Ok(signature) => {
                        Ok(public_key.verify_strict(message, &signature).is_ok())
                    },
                    Err(_) => Ok(false),
                }
            },
            Err(_) => Ok(false),
        }
    }

    fn sig_type(&self) -> &'static str {
        "ed25519"
    }
}

impl AccSignature for BTCSignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        // Non-standard signature type - serialization/deserialization only
        // TODO: Stage 1.4 will implement proper verification logic
        Ok(false)
    }

    fn sig_type(&self) -> &'static str {
        "btc"
    }
}

impl AccSignature for BTCLegacySignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        // Non-standard signature type - serialization/deserialization only
        // TODO: Stage 1.4 will implement proper verification logic
        Ok(false)
    }

    fn sig_type(&self) -> &'static str {
        "btcLegacy"
    }
}

impl AccSignature for ETHSignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        // ECDSA/Ethereum signature verification
        use k256::ecdsa::{Signature as EcdsaSignature, VerifyingKey};
        use k256::elliptic_curve::sec1::ToEncodedPoint;
        use sha2::{Sha256, Digest};

        // TODO: Implement proper ECDSA verification
        // For now, basic structure validation
        if self.signature.is_empty() || self.public_key.is_empty() {
            return Ok(false);
        }

        // Placeholder - Stage 1.4 will add proper crypto vectors
        Ok(false)
    }

    fn sig_type(&self) -> &'static str {
        "eth"
    }
}

impl AccSignature for RsaSha256Signature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        // RSA signature verification - using simplified imports
        // TODO: Implement proper RSA verification in Stage 1.4
        // For now, basic structure validation
        if self.signature.is_empty() || self.public_key.is_empty() {
            return Ok(false);
        }

        // Placeholder - Stage 1.4 will add proper crypto vectors
        Ok(false)
    }

    fn sig_type(&self) -> &'static str {
        "rsaSha256"
    }
}

impl AccSignature for EcdsaSha256Signature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        // ECDSA/Ethereum signature verification
        use k256::ecdsa::{Signature as EcdsaSignature, VerifyingKey};
        use k256::elliptic_curve::sec1::ToEncodedPoint;
        use sha2::{Sha256, Digest};

        // TODO: Implement proper ECDSA verification
        // For now, basic structure validation
        if self.signature.is_empty() || self.public_key.is_empty() {
            return Ok(false);
        }

        // Placeholder - Stage 1.4 will add proper crypto vectors
        Ok(false)
    }

    fn sig_type(&self) -> &'static str {
        "ecdsaSha256"
    }
}

impl AccSignature for TypedDataSignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        // Non-standard signature type - serialization/deserialization only
        // TODO: Stage 1.4 will implement proper verification logic
        Ok(false)
    }

    fn sig_type(&self) -> &'static str {
        "typedData"
    }
}

impl AccSignature for ReceiptSignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        // Non-standard signature type - serialization/deserialization only
        // TODO: Stage 1.4 will implement proper verification logic
        Ok(false)
    }

    fn sig_type(&self) -> &'static str {
        "receipt"
    }
}

impl AccSignature for PartitionSignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        // Non-standard signature type - serialization/deserialization only
        // TODO: Stage 1.4 will implement proper verification logic
        Ok(false)
    }

    fn sig_type(&self) -> &'static str {
        "partition"
    }
}

impl AccSignature for SignatureSet {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        // Non-standard signature type - serialization/deserialization only
        // TODO: Stage 1.4 will implement proper verification logic
        Ok(false)
    }

    fn sig_type(&self) -> &'static str {
        "signatureSet"
    }
}

impl AccSignature for RemoteSignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        // Non-standard signature type - serialization/deserialization only
        // TODO: Stage 1.4 will implement proper verification logic
        Ok(false)
    }

    fn sig_type(&self) -> &'static str {
        "remote"
    }
}

impl AccSignature for DelegatedSignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        // Non-standard signature type - serialization/deserialization only
        // TODO: Stage 1.4 will implement proper verification logic
        Ok(false)
    }

    fn sig_type(&self) -> &'static str {
        "delegated"
    }
}

impl AccSignature for InternalSignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        // Non-standard signature type - serialization/deserialization only
        // TODO: Stage 1.4 will implement proper verification logic
        Ok(false)
    }

    fn sig_type(&self) -> &'static str {
        "internal"
    }
}

impl AccSignature for AuthoritySignature {
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {
        
        // Non-standard signature type - serialization/deserialization only
        // TODO: Stage 1.4 will implement proper verification logic
        Ok(false)
    }

    fn sig_type(&self) -> &'static str {
        "authority"
    }
}