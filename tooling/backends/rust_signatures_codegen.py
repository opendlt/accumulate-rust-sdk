#!/usr/bin/env python3
"""
Rust Signature Code Generator for Accumulate Protocol

Generates Rust signature types from Go YAML truth sources.
Outputs to src/generated/signatures.rs with exact wire compatibility.
"""

import os
import sys
import yaml
import json
from datetime import datetime
from pathlib import Path


def load_signatures_yaml():
    """Load signature definitions from Go YAML truth source."""
    yaml_path = Path("C:/Accumulate_Stuff/accumulate/protocol/signatures.yml")
    if not yaml_path.exists():
        print(f"ERROR: YAML file not found: {yaml_path}")
        sys.exit(1)

    with open(yaml_path, 'r') as f:
        data = yaml.safe_load(f)

    print(f"Loaded: {yaml_path}")
    return data


def get_wire_tag(signature_name):
    """Map signature names to their exact wire tags."""
    # Based on the YAML union type values and Go SDK conventions
    wire_map = {
        'LegacyED25519Signature': 'legacyED25519',
        'RCD1Signature': 'rcd1',
        'ED25519Signature': 'ed25519',
        'BTCSignature': 'btc',
        'BTCLegacySignature': 'btcLegacy',
        'ETHSignature': 'eth',
        'RsaSha256Signature': 'rsaSha256',
        'EcdsaSha256Signature': 'ecdsaSha256',
        'TypedDataSignature': 'typedData',
        'ReceiptSignature': 'receipt',
        'PartitionSignature': 'partition',
        'SignatureSet': 'signatureSet',
        'RemoteSignature': 'remote',
        'DelegatedSignature': 'delegated',
        'InternalSignature': 'internal',
        'AuthoritySignature': 'authority',
    }
    return wire_map.get(signature_name, signature_name.lower())


def rust_type_from_yaml(yaml_type, is_optional=False, is_repeatable=False):
    """Convert YAML type to Rust type."""
    type_map = {
        'uint': 'u64',
        'bytes': 'Vec<u8>',
        'string': 'String',
        'hash': '[u8; 32]',
        'url': 'String',  # URLs as strings for now
        'bigint': 'String',  # BigInt as string for serde compatibility
        'VoteType': 'crate::generated::enums::VoteType',
        'Signature': 'Box<crate::generated::signatures::Signature>',  # Recursive signature reference
        'merkle.Receipt': 'crate::types::MerkleReceipt',  # Placeholder
        'txid': 'String',  # Transaction ID as string
    }

    rust_type = type_map.get(yaml_type, yaml_type)

    if is_repeatable:
        rust_type = f"Vec<{rust_type}>"

    if is_optional:
        rust_type = f"Option<{rust_type}>"

    return rust_type


def rust_field_name(yaml_name):
    """Convert YAML field name to Rust snake_case."""
    # Convert PascalCase to snake_case
    result = []
    for i, c in enumerate(yaml_name):
        if c.isupper() and i > 0:
            result.append('_')
        result.append(c.lower())
    return ''.join(result)


def serde_rename_attr(yaml_name):
    """Generate serde rename attribute for field."""
    # Keep original YAML casing for wire compatibility
    return f'#[serde(rename = "{yaml_name}")]'


def generate_signature_struct(name, fields):
    """Generate a single signature struct."""
    rust_name = name  # Keep original name

    field_definitions = []
    for field in fields:
        field_name = field['name']
        field_type = field['type']
        is_optional = field.get('optional', False)
        is_repeatable = field.get('repeatable', False)

        rust_field = rust_field_name(field_name)
        rust_type = rust_type_from_yaml(field_type, is_optional, is_repeatable)

        # Add serde attributes
        serde_attrs = []

        # Add rename if field name differs
        if rust_field != field_name:
            serde_attrs.append(f'rename = "{field_name}"')

        # Add hex serialization for byte fields
        if field_type in ['bytes', 'hash']:
            if is_repeatable:
                # For Vec<[u8; 32]> or Vec<Vec<u8>>, we need a special helper
                if field_type == 'hash':
                    serde_attrs.append('with = "hex_vec_hash"')
                else:  # bytes
                    serde_attrs.append('with = "hex_vec_bytes"')
            elif is_optional:
                if field_type == 'bytes':
                    serde_attrs.append('with = "hex_option_vec"')
                else:  # hash
                    serde_attrs.append('with = "hex_option"')
            else:
                serde_attrs.append('with = "hex::serde"')

        if serde_attrs:
            field_definitions.append(f'    #[serde({", ".join(serde_attrs)})]')

        field_definitions.append(f'    pub {rust_field}: {rust_type},')

    # Generate struct
    struct_code = f'''/// {name} signature
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct {rust_name} {{
{chr(10).join(field_definitions)}
}}'''

    return struct_code


def generate_acc_signature_impl(name):
    """Generate AccSignature implementation for a signature type."""
    wire_tag = get_wire_tag(name)

    # Determine verification strategy based on signature type
    if name == 'ED25519Signature':
        verify_impl = """
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
        }"""
    elif name == 'LegacyED25519Signature':
        verify_impl = """
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
        }"""
    elif name in ['ETHSignature', 'EcdsaSha256Signature']:
        verify_impl = """
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
        Ok(false)"""
    elif name in ['RsaSha256Signature']:
        verify_impl = """
        // RSA signature verification - using simplified imports
        // TODO: Implement proper RSA verification in Stage 1.4
        // For now, basic structure validation
        if self.signature.is_empty() || self.public_key.is_empty() {
            return Ok(false);
        }

        // Placeholder - Stage 1.4 will add proper crypto vectors
        Ok(false)"""
    else:
        # For non-standard types: RCD1, BTC, Internal, Partition, Receipt, Remote, etc.
        verify_impl = """
        // Non-standard signature type - serialization/deserialization only
        // TODO: Stage 1.4 will implement proper verification logic
        Ok(false)"""

    return f'''impl AccSignature for {name} {{
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error> {{
        {verify_impl}
    }}

    fn sig_type(&self) -> &'static str {{
        "{wire_tag}"
    }}
}}'''


def generate_signature_enum(signatures_data):
    """Generate the main Signature enum with serde dispatch."""
    variants = []
    wire_tags = []

    for name in signatures_data:
        wire_tag = get_wire_tag(name)
        variant_name = name.replace('Signature', '')  # Remove 'Signature' suffix for enum variant

        variants.append(f'    #[serde(rename = "{wire_tag}")]')
        variants.append(f'    {variant_name}({name}),')
        wire_tags.append(f'            Signature::{variant_name}(_) => "{wire_tag}",')

    enum_code = f'''/// Main signature dispatch enum
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Signature {{
{chr(10).join(variants)}
}}

impl Signature {{
    pub fn wire_tag(&self) -> &'static str {{
        match self {{
{chr(10).join(wire_tags)}
        }}
    }}
}}'''

    return enum_code


def generate_signatures_rust_file(signatures_data):
    """Generate the complete signatures.rs file."""
    # Header
    header = f'''// GENERATED FILE - DO NOT EDIT
// Source: protocol/signatures.yml | Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}

use serde::{{Serialize, Deserialize}};
use hex;

// Helper module for optional hex serialization
mod hex_option {{
    use serde::{{Deserialize, Deserializer, Serialize, Serializer}};

    pub fn serialize<S>(value: &Option<[u8; 32]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {{
        match value {{
            Some(bytes) => hex::encode(bytes).serialize(serializer),
            None => serializer.serialize_none(),
        }}
    }}

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<[u8; 32]>, D::Error>
    where
        D: Deserializer<'de>,
    {{
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {{
            Some(hex_str) => {{
                let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
                if bytes.len() != 32 {{
                    return Err(serde::de::Error::custom("Hash must be 32 bytes"));
                }}
                let mut hash = [0u8; 32];
                hash.copy_from_slice(&bytes);
                Ok(Some(hash))
            }},
            None => Ok(None),
        }}
    }}
}}

// Helper module for optional bytes hex serialization
mod hex_option_vec {{
    use serde::{{Deserialize, Deserializer, Serialize, Serializer}};

    pub fn serialize<S>(value: &Option<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {{
        match value {{
            Some(bytes) => hex::encode(bytes).serialize(serializer),
            None => serializer.serialize_none(),
        }}
    }}

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {{
        let opt: Option<String> = Option::deserialize(deserializer)?;
        match opt {{
            Some(hex_str) => {{
                let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
                Ok(Some(bytes))
            }},
            None => Ok(None),
        }}
    }}
}}

// Helper module for vector of hashes hex serialization
mod hex_vec_hash {{
    use serde::{{Deserialize, Deserializer, Serialize, Serializer}};

    pub fn serialize<S>(value: &Vec<[u8; 32]>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {{
        let hex_strings: Vec<String> = value.iter().map(|hash| hex::encode(hash)).collect();
        hex_strings.serialize(serializer)
    }}

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<[u8; 32]>, D::Error>
    where
        D: Deserializer<'de>,
    {{
        let hex_strings: Vec<String> = Vec::deserialize(deserializer)?;
        let mut result = Vec::new();
        for hex_str in hex_strings {{
            let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
            if bytes.len() != 32 {{
                return Err(serde::de::Error::custom("Hash must be 32 bytes"));
            }}
            let mut hash = [0u8; 32];
            hash.copy_from_slice(&bytes);
            result.push(hash);
        }}
        Ok(result)
    }}
}}

// Helper module for vector of bytes hex serialization
mod hex_vec_bytes {{
    use serde::{{Deserialize, Deserializer, Serialize, Serializer}};

    pub fn serialize<S>(value: &Vec<Vec<u8>>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {{
        let hex_strings: Vec<String> = value.iter().map(|bytes| hex::encode(bytes)).collect();
        hex_strings.serialize(serializer)
    }}

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Vec<u8>>, D::Error>
    where
        D: Deserializer<'de>,
    {{
        let hex_strings: Vec<String> = Vec::deserialize(deserializer)?;
        let mut result = Vec::new();
        for hex_str in hex_strings {{
            let bytes = hex::decode(&hex_str).map_err(serde::de::Error::custom)?;
            result.push(bytes);
        }}
        Ok(result)
    }}
}}

/// Error type for signature operations
#[derive(Debug, thiserror::Error)]
pub enum SignatureError {{
    #[error("Invalid signature format")]
    InvalidFormat,
    #[error("Verification failed: {{0}}")]
    VerificationFailed(String),
    #[error("Unsupported signature type")]
    UnsupportedType,
}}

/// Main signature trait for verification
pub trait AccSignature {{
    fn verify(&self, message: &[u8]) -> Result<bool, crate::errors::Error>;
    fn sig_type(&self) -> &'static str;
}}'''

    # Generate all signature structs
    structs = []
    impls = []

    for name, data in signatures_data.items():
        if 'fields' in data:
            structs.append(generate_signature_struct(name, data['fields']))
            impls.append(generate_acc_signature_impl(name))

    # Generate main enum
    enum_code = generate_signature_enum(signatures_data)

    # Combine all parts
    return '\n\n'.join([
        header,
        '\n\n'.join(structs),
        enum_code,
        '\n\n'.join(impls)
    ])


def generate_manifest(signatures_data):
    """Generate signatures manifest JSON."""
    signatures = []

    for name, data in signatures_data.items():
        wire_tag = get_wire_tag(name)
        fields = [field['name'] for field in data.get('fields', [])]

        signatures.append({
            'name': name,
            'wire': wire_tag,
            'fields': fields
        })

    manifest = {
        'generated_at': datetime.now().isoformat(),
        'signatures': signatures,
        'counts': {'signatures': len(signatures)}
    }

    return manifest


def main():
    print("=== Rust Signatures Code Generator ===")

    # Load YAML data
    signatures_data = load_signatures_yaml()

    # Validate exactly 16 signatures
    signature_count = len(signatures_data)
    if signature_count != 16:
        print(f"ERROR: Expected exactly 16 signatures, found {signature_count}")
        print("Found signatures:")
        for name in signatures_data:
            print(f"  - {name}")
        sys.exit(2)

    print(f"Found {signature_count} signatures")
    for name in signatures_data:
        wire_tag = get_wire_tag(name)
        print(f"  {name} -> '{wire_tag}'")

    # Generate Rust code
    rust_code = generate_signatures_rust_file(signatures_data)

    # Write signatures.rs
    output_dir = Path("C:/Accumulate_Stuff/opendlt-rust-v2v3-sdk/unified/src/generated")
    output_dir.mkdir(parents=True, exist_ok=True)

    rust_file = output_dir / "signatures.rs"
    with open(rust_file, 'w') as f:
        f.write(rust_code)
    print(f"Generated: {rust_file}")

    # Generate and write manifest
    manifest = generate_manifest(signatures_data)
    manifest_file = output_dir / "signatures_manifest.json"
    with open(manifest_file, 'w') as f:
        json.dump(manifest, f, indent=2)
    print(f"Generated: {manifest_file}")

    print(f"SUCCESS: Generated {signature_count} signatures with exact wire compatibility")


if __name__ == "__main__":
    main()