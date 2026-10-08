//! GENERATED FILE - DO NOT EDIT
//! Source: protocol/transaction.yml
//! Generated: 2025-10-03 22:05:19
//!
//! NOTE: `HashLockOptions` and `TransactionHeader::hash_lock` (Accumulate 1.4.6.x) were added by
//! hand. `tooling/backends/rust_tx_header_codegen.py` does not know the type yet, so regenerating
//! this file would drop them; teach the generator first.

#![allow(missing_docs)]

use serde::{Serialize, Deserialize};


mod hex_option_vec {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match value {
            Some(bytes) => serializer.serialize_str(&hex::encode(bytes)),
            None => serializer.serialize_none(),
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::Error;
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {
            Some(hex_str) => {
                hex::decode(&hex_str).map(Some).map_err(D::Error::custom)
            }
            None => Ok(None),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExpireOptions {
    pub at_time: Option<u64>,
}

impl ExpireOptions {
    pub fn validate(&self) -> Result<(), crate::errors::Error> {
        // If at_time is provided, it should be a reasonable timestamp
        // (not in the distant past, and not impossibly far in the future)
        if let Some(at_time) = self.at_time {
            // Timestamps before year 2000 are likely errors (Unix timestamp 946684800)
            const YEAR_2000_UNIX: u64 = 946684800;
            // Timestamps more than 100 years in the future are likely errors
            const HUNDRED_YEARS_SECONDS: u64 = 100 * 365 * 24 * 60 * 60;

            if at_time > 0 && at_time < YEAR_2000_UNIX {
                return Err(crate::errors::ValidationError::OutOfRange {
                    field: "atTime".to_string(),
                    min: YEAR_2000_UNIX.to_string(),
                    max: "far future".to_string(),
                }.into());
            }

            // Get current time estimate (we can't use std::time here due to no_std compatibility concerns,
            // but we can at least check for obviously invalid future values)
            if at_time > YEAR_2000_UNIX + HUNDRED_YEARS_SECONDS {
                return Err(crate::errors::ValidationError::OutOfRange {
                    field: "atTime".to_string(),
                    min: "now".to_string(),
                    max: "100 years from epoch".to_string(),
                }.into());
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HoldUntilOptions {
    pub minor_block: Option<u64>,
}

impl HoldUntilOptions {
    pub fn validate(&self) -> Result<(), crate::errors::Error> {
        // If minor_block is provided, it should be positive (block numbers start at 1)
        if let Some(minor_block) = self.minor_block {
            if minor_block == 0 {
                return Err(crate::errors::ValidationError::InvalidFieldValue {
                    field: "minorBlock".to_string(),
                    reason: "minor block number must be greater than zero".to_string(),
                }.into());
            }
        }
        Ok(())
    }
}

/// Hash lock conditions for a transaction (TransactionHeader field 8,
/// HashLockOptions in protocol/transaction.yml, Accumulate 1.4.6.7).
///
/// `expiration` is an RFC 3339 UTC timestamp string (the Go JSON form), e.g.
/// `2030-01-02T03:04:05Z`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HashLockOptions {
    pub hash_algorithm: crate::generated::enums::HashAlgorithm,
    #[serde(with = "hex::serde")]
    pub hash: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub expiration: Option<String>,
}

impl HashLockOptions {
    /// Build options from an algorithm, hash and optional expiration (Unix seconds).
    pub fn new(
        hash_algorithm: crate::generated::enums::HashAlgorithm,
        hash: Vec<u8>,
        expiration_unix: Option<i64>,
    ) -> Self {
        let expiration = expiration_unix.and_then(|t| {
            chrono::DateTime::from_timestamp(t, 0)
                .map(|dt| dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        });
        Self { hash_algorithm, hash, expiration }
    }

    /// Expiration as Unix seconds, if set.
    pub fn expiration_unix(&self) -> Result<Option<i64>, crate::errors::Error> {
        match &self.expiration {
            None => Ok(None),
            Some(s) => chrono::DateTime::parse_from_rfc3339(s)
                .map(|dt| Some(dt.timestamp()))
                .map_err(|e| crate::errors::ValidationError::InvalidFieldValue {
                    field: "hashLock.expiration".to_string(),
                    reason: format!("invalid RFC 3339 timestamp: {}", e),
                }.into()),
        }
    }

    /// Binary form used for header field 8.
    pub fn to_binary(&self) -> Result<crate::codec::signing::HashLockBinary, crate::errors::Error> {
        Ok(crate::codec::signing::HashLockBinary {
            hash_algorithm: self.hash_algorithm.value(),
            hash: self.hash.clone(),
            expiration: self.expiration_unix()?,
        })
    }

    /// Everything [`validate`](Self::validate) checks, plus the node's expiration window: the lock
    /// must expire between 10 minutes and 30 days after `now_unix`, and must have an expiration.
    pub fn validate_for_submit(&self, now_unix: i64) -> Result<(), crate::errors::Error> {
        self.validate()?;
        let invalid = |reason: &str| -> crate::errors::Error {
            crate::errors::ValidationError::InvalidFieldValue {
                field: "hashLock.expiration".to_string(),
                reason: reason.to_string(),
            }
            .into()
        };
        match self.expiration_unix()? {
            None => Err(invalid("expiration is required")),
            Some(exp) if exp - now_unix < 10 * 60 => {
                Err(invalid("expiration must be at least 10 minutes in the future"))
            }
            Some(exp) if exp - now_unix > 30 * 24 * 3600 => {
                Err(invalid("expiration must be at most 30 days in the future"))
            }
            Some(_) => Ok(()),
        }
    }

    pub fn validate(&self) -> Result<(), crate::errors::Error> {
        use crate::generated::enums::HashAlgorithm;
        let expected = match self.hash_algorithm {
            HashAlgorithm::Unknown => {
                return Err(crate::errors::ValidationError::InvalidFieldValue {
                    field: "hashLock.hashAlgorithm".to_string(),
                    reason: "hash algorithm must be set".to_string(),
                }.into());
            }
            HashAlgorithm::Sha256 | HashAlgorithm::Sha256D => 32,
            HashAlgorithm::Hash160 => 20,
        };
        if self.hash.len() != expected {
            return Err(crate::errors::ValidationError::InvalidFieldValue {
                field: "hashLock.hash".to_string(),
                reason: format!("hash must be {} bytes for {:?}, got {}", expected, self.hash_algorithm, self.hash.len()),
            }.into());
        }
        self.expiration_unix()?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionHeader {
    pub principal: String,
    #[serde(with = "hex::serde")]
    pub initiator: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub memo: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[serde(with = "hex_option_vec")]
    pub metadata: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub expire: Option<ExpireOptions>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub hold_until: Option<HoldUntilOptions>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub authorities: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub hash_lock: Option<HashLockOptions>,
}

impl TransactionHeader {
    /// Field-level validation aligned with YAML truth
    pub fn validate(&self) -> Result<(), crate::errors::Error> {
        if self.principal.is_empty() { return Err(crate::errors::Error::General("Principal URL cannot be empty".to_string())); }

        // Validate principal URL contains only ASCII characters
        if !self.principal.is_ascii() {
            return Err(crate::errors::Error::General("Principal URL must contain only ASCII characters".to_string()));
        }

        // Validate initiator size (reasonable limit: 32KB)
        const MAX_INITIATOR_SIZE: usize = 32 * 1024;
        if self.initiator.len() > MAX_INITIATOR_SIZE {
            return Err(crate::errors::Error::General(format!("Initiator size {} exceeds maximum of {}", self.initiator.len(), MAX_INITIATOR_SIZE)));
        }

        // Validate authorities (Additional Authorities)
        if let Some(ref authorities) = self.authorities {
            self.validate_authorities(authorities)?;
        }

        // Validate metadata - no null bytes allowed in binary metadata
        if let Some(ref metadata) = self.metadata {
            if metadata.contains(&0) {
                return Err(crate::errors::Error::General("Metadata cannot contain null bytes".to_string()));
            }
        }

        if let Some(ref opts) = self.expire { opts.validate()?; }
        if let Some(ref opts) = self.hold_until { opts.validate()?; }
        if let Some(ref opts) = self.hash_lock { opts.validate()?; }
        Ok(())
    }

    /// Validate additional authorities list
    fn validate_authorities(&self, authorities: &[String]) -> Result<(), crate::errors::Error> {
        // Maximum number of additional authorities per Go protocol limits
        const MAX_AUTHORITIES: usize = 20;

        if authorities.len() > MAX_AUTHORITIES {
            return Err(crate::errors::ValidationError::InvalidFieldValue {
                field: "authorities".to_string(),
                reason: format!("too many authorities: {} (max {})", authorities.len(), MAX_AUTHORITIES),
            }.into());
        }

        for (index, authority) in authorities.iter().enumerate() {
            // Authority URL cannot be empty
            if authority.is_empty() {
                return Err(crate::errors::ValidationError::InvalidFieldValue {
                    field: format!("authorities[{}]", index),
                    reason: "authority URL cannot be empty".to_string(),
                }.into());
            }

            // Authority URL must start with acc://
            if !authority.starts_with("acc://") {
                return Err(crate::errors::ValidationError::InvalidUrl(
                    format!("authorities[{}]: must start with 'acc://', got '{}'", index, authority)
                ).into());
            }

            // Authority URL must contain only ASCII characters
            if !authority.is_ascii() {
                return Err(crate::errors::ValidationError::InvalidUrl(
                    format!("authorities[{}]: URL must contain only ASCII characters", index)
                ).into());
            }

            // Authority URL must not contain whitespace
            if authority.chars().any(|c| c.is_whitespace()) {
                return Err(crate::errors::ValidationError::InvalidUrl(
                    format!("authorities[{}]: URL must not contain whitespace", index)
                ).into());
            }

            // Check for reasonable URL length
            const MAX_URL_LENGTH: usize = 1024;
            if authority.len() > MAX_URL_LENGTH {
                return Err(crate::errors::ValidationError::InvalidUrl(
                    format!("authorities[{}]: URL too long ({} > {})", index, authority.len(), MAX_URL_LENGTH)
                ).into());
            }

            // Authority should point to a key book (must contain /book/ or /page/ pattern)
            // This is advisory - some authorities may be identities themselves
            // Relaxed validation: just ensure it's a valid Accumulate URL structure
            let url_path = &authority[6..]; // Skip "acc://"
            if url_path.is_empty() || url_path == "/" {
                return Err(crate::errors::ValidationError::InvalidUrl(
                    format!("authorities[{}]: URL has no identity", index)
                ).into());
            }
        }

        // Check for duplicate authorities
        let mut seen = std::collections::HashSet::new();
        for (index, authority) in authorities.iter().enumerate() {
            let normalized = authority.to_lowercase();
            if !seen.insert(normalized.clone()) {
                return Err(crate::errors::ValidationError::InvalidFieldValue {
                    field: format!("authorities[{}]", index),
                    reason: format!("duplicate authority URL: {}", authority),
                }.into());
            }
        }

        Ok(())
    }
}