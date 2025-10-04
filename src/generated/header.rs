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
    #[serde(rename = "AtTime")]
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
    #[serde(rename = "MinorBlock")]
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
    #[serde(rename = "Principal")]
    pub principal: String,
    #[serde(rename = "Initiator")]
    #[serde(with = "hex::serde")]
    pub initiator: Vec<u8>,
    #[serde(rename = "Memo")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub memo: Option<String>,
    #[serde(rename = "Metadata")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    #[serde(with = "hex_option_vec")]
    pub metadata: Option<Vec<u8>>,
    #[serde(rename = "Expire")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub expire: Option<ExpireOptions>,
    #[serde(rename = "HoldUntil")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub hold_until: Option<HoldUntilOptions>,
    #[serde(rename = "Authorities")]
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub authorities: Option<Vec<String>>,
}

impl TransactionHeader {
    /// Field-level validation aligned with YAML truth
    pub fn validate(&self) -> Result<(), crate::errors::Error> {
        if self.principal.is_empty() { return Err(crate::errors::Error::General("Principal URL cannot be empty".to_string())); }
        if let Some(ref opts) = self.expire { opts.validate()?; }
        if let Some(ref opts) = self.hold_until { opts.validate()?; }
        Ok(())
    }
}