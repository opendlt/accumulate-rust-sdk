//! GENERATED FILE - DO NOT EDIT
//! Source: protocol/transaction.yml
//! Generated: 2025-10-03 22:05:19

use serde::{Serialize, Deserialize};


mod hex_option_vec {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

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
        // TODO: Add specific validation logic for ExpireOptions
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
        // TODO: Add specific validation logic for HoldUntilOptions
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

        // Validate authorities - no empty authority URLs allowed
        if let Some(ref authorities) = self.authorities {
            for authority in authorities {
                if authority.is_empty() {
                    return Err(crate::errors::Error::General("Authority URL cannot be empty".to_string()));
                }
            }
        }

        // Validate metadata - no null bytes allowed in binary metadata
        if let Some(ref metadata) = self.metadata {
            if metadata.contains(&0) {
                return Err(crate::errors::Error::General("Metadata cannot contain null bytes".to_string()));
            }
        }

        if let Some(ref opts) = self.expire { opts.validate()?; }
        if let Some(ref opts) = self.hold_until { opts.validate()?; }
        Ok(())
    }
}