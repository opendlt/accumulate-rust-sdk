#!/usr/bin/env python3
"""
TransactionHeader Code Generator for Rust Accumulate SDK

Generates src/generated/header.rs from Go YAML sources.
Canonical truth: protocol/transaction.yml

Requirements:
- Extract exact TransactionHeader field schema from YAML
- Generate Rust struct with proper field types and nullability
- Add validation method with runtime checks
- Create manifest JSON with metadata
"""

import yaml
import json
import os
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any, Optional

# Constants
GO_REPO = Path(r"C:\Accumulate_Stuff\accumulate")
RUST_ROOT = Path(r"C:\Accumulate_Stuff\opendlt-rust-v2v3-sdk\unified")
SRC_DIR = RUST_ROOT / "src"
GEN_DIR = SRC_DIR / "generated"

class HeaderField:
    """Represents a parsed header field from YAML"""

    def __init__(self, name: str, field_def: Dict[str, Any]):
        self.name = name
        self.yaml_type = field_def.get('type', 'unknown')
        self.optional = field_def.get('optional', False)
        self.repeatable = field_def.get('repeatable', False)
        self.description = field_def.get('description', '')
        self.pointer = field_def.get('pointer', False)

    def get_rust_type(self) -> str:
        """Convert YAML type to Rust type"""
        type_mapping = {
            'url': 'String',
            'hash': 'Vec<u8>',
            'string': 'String',
            'bytes': 'Vec<u8>',
            'uint': 'u64',
            'uvarint': 'u64',
            'time': 'u64',
            'bool': 'bool',
            'ExpireOptions': 'ExpireOptions',
            'HoldUntilOptions': 'HoldUntilOptions',
        }

        base_type = type_mapping.get(self.yaml_type, 'serde_json::Value')

        if self.repeatable:
            base_type = f"Vec<{base_type}>"

        if self.optional:
            base_type = f"Option<{base_type}>"

        return base_type

    def get_field_info(self) -> Dict[str, Any]:
        """Get field information for manifest"""
        return {
            'name': self.name,
            'type': self.yaml_type,
            'required': not self.optional,
            'repeatable': self.repeatable
        }

def load_yaml_file(file_path: Path) -> Dict[str, Any]:
    """Load a YAML file and return its contents"""
    if not file_path.exists():
        print(f"WARN File not found: {file_path}")
        return {}

    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = yaml.safe_load(f)
            print(f"OK Loaded {file_path}")
            return content or {}
    except yaml.YAMLError as e:
        print(f"ERROR loading {file_path}: {e}")
        return {}

def extract_transaction_header(yaml_data: Dict[str, Any]) -> List[HeaderField]:
    """Extract TransactionHeader fields from YAML data"""
    if 'TransactionHeader' not in yaml_data:
        print("ERROR TransactionHeader not found in YAML")
        sys.exit(2)

    header_def = yaml_data['TransactionHeader']
    fields_list = header_def.get('fields', [])

    if not fields_list:
        print("ERROR TransactionHeader has no fields")
        sys.exit(2)

    fields = []
    for field_def in fields_list:
        name = field_def.get('name')
        if name:
            field = HeaderField(name, field_def)
            fields.append(field)
            print(f"  Found field: {name} ({field.yaml_type}, required: {not field.optional})")

    return fields

def extract_nested_types(yaml_data: Dict[str, Any]) -> Dict[str, List[HeaderField]]:
    """Extract nested type definitions (ExpireOptions, HoldUntilOptions)"""
    nested_types = {}

    for type_name in ['ExpireOptions', 'HoldUntilOptions']:
        if type_name in yaml_data:
            type_def = yaml_data[type_name]
            fields_list = type_def.get('fields', [])

            fields = []
            for field_def in fields_list:
                name = field_def.get('name')
                if name:
                    field = HeaderField(name, field_def)
                    fields.append(field)

            nested_types[type_name] = fields
            print(f"  Found nested type: {type_name} with {len(fields)} fields")

    return nested_types

def generate_nested_type_structs(nested_types: Dict[str, List[HeaderField]]) -> str:
    """Generate Rust structs for nested types"""
    structs = []

    for type_name, fields in nested_types.items():
        field_lines = []
        for field in fields:
            rust_type = field.get_rust_type()
            snake_case_name = camel_to_snake_case(field.name)

            # Add serde rename if needed
            if snake_case_name != field.name:
                field_lines.append(f'    #[serde(rename = "{field.name}")]')

            field_lines.append(f'    pub {snake_case_name}: {rust_type},')

        fields_str = '\n'.join(field_lines) if field_lines else '    // No fields'

        struct_code = f"""#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct {type_name} {{
{fields_str}
}}

impl {type_name} {{
    pub fn validate(&self) -> Result<(), crate::errors::Error> {{
        // TODO: Add specific validation logic for {type_name}
        Ok(())
    }}
}}"""
        structs.append(struct_code)

    return '\n\n'.join(structs)

def camel_to_snake_case(name: str) -> str:
    """Convert camelCase to snake_case"""
    import re
    s1 = re.sub('([a-z0-9])([A-Z])', r'\1_\2', name)
    return s1.lower()

def generate_header_struct(fields: List[HeaderField]) -> str:
    """Generate the TransactionHeader Rust struct"""
    field_lines = []

    for field in fields:
        rust_type = field.get_rust_type()
        snake_case_name = camel_to_snake_case(field.name)

        # Add serde rename if needed
        if snake_case_name != field.name:
            field_lines.append(f'    #[serde(rename = "{field.name}")]')

        # Add skip_serializing_if and default for optional fields
        if field.optional:
            field_lines.append(f'    #[serde(skip_serializing_if = "Option::is_none", default)]')

        # Add hex serialization for Vec<u8> fields
        if field.yaml_type in ['hash', 'bytes'] and 'Vec<u8>' in rust_type:
            if field.optional:
                field_lines.append(f'    #[serde(with = "hex_option_vec")]')
            else:
                field_lines.append(f'    #[serde(with = "hex::serde")]')

        field_lines.append(f'    pub {snake_case_name}: {rust_type},')

    fields_str = '\n'.join(field_lines)

    return f"""#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransactionHeader {{
{fields_str}
}}"""

def generate_validation_impl(fields: List[HeaderField]) -> str:
    """Generate validation implementation for TransactionHeader"""
    validation_checks = []

    for field in fields:
        snake_case_name = camel_to_snake_case(field.name)

        # Add specific validations based on field type and constraints
        if field.yaml_type == 'url' and not field.optional:
            validation_checks.append(f'        if self.{snake_case_name}.is_empty() {{ '
                                   f'return Err(crate::errors::Error::General("Principal URL cannot be empty".to_string())); }}')

        # Add validation for nested types
        if field.yaml_type in ['ExpireOptions', 'HoldUntilOptions']:
            if field.optional:
                validation_checks.append(f'        if let Some(ref opts) = self.{snake_case_name} {{ opts.validate()?; }}')
            else:
                validation_checks.append(f'        self.{snake_case_name}.validate()?;')

    # If no specific validations, add a generic success
    if not validation_checks:
        validation_checks.append('        // TODO: Add field-specific validation constraints from YAML')

    validation_body = '\n'.join(validation_checks)

    return f"""impl TransactionHeader {{
    /// Field-level validation aligned with YAML truth
    pub fn validate(&self) -> Result<(), crate::errors::Error> {{
{validation_body}
        Ok(())
    }}
}}"""

def generate_hex_helpers() -> str:
    """Generate hex serialization helper modules"""
    return """
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
"""

def generate_header_rs(fields: List[HeaderField], nested_types: Dict[str, List[HeaderField]]) -> str:
    """Generate the complete header.rs file"""
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    header = f"""//! GENERATED FILE - DO NOT EDIT
//! Source: protocol/transaction.yml
//! Generated: {timestamp}

use serde::{{Serialize, Deserialize}};

"""

    # Add hex helpers if needed
    needs_hex_helpers = any(
        field.yaml_type in ['hash', 'bytes'] and field.optional
        for field in fields
    )
    if needs_hex_helpers:
        header += generate_hex_helpers()

    # Generate nested type structs first
    nested_structs = generate_nested_type_structs(nested_types)
    if nested_structs:
        header += nested_structs + "\n\n"

    # Generate main header struct
    header_struct = generate_header_struct(fields)
    validation_impl = generate_validation_impl(fields)

    return header + header_struct + "\n\n" + validation_impl

def generate_manifest(fields: List[HeaderField], nested_types: Dict[str, List[HeaderField]]) -> Dict[str, Any]:
    """Generate the header manifest JSON"""
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    field_info = []
    for field in fields:
        field_info.append(field.get_field_info())

    nested_info = {}
    for type_name, type_fields in nested_types.items():
        nested_info[type_name] = [field.get_field_info() for field in type_fields]

    return {
        'generated_at': timestamp,
        'struct': 'TransactionHeader',
        'fields': field_info,
        'nested_types': nested_info
    }

def validate_required_fields(fields: List[HeaderField]) -> None:
    """Validate that all expected header fields are present"""
    expected_fields = {
        'Principal', 'Initiator', 'Memo', 'Metadata',
        'Expire', 'HoldUntil', 'Authorities'
    }

    found_fields = {field.name for field in fields}
    missing_fields = expected_fields - found_fields
    extra_fields = found_fields - expected_fields

    if missing_fields or extra_fields:
        print("\nHEADER_FIELD_MISMATCH:")
        if missing_fields:
            print(f"Missing: {', '.join(sorted(missing_fields))}")
        if extra_fields:
            print(f"Extra: {', '.join(sorted(extra_fields))}")
        print(f"Expected: {len(expected_fields)}, Found: {len(found_fields)}")
        # Continue for now, but warn about field mismatches
        print("WARN Continuing with discovered fields...")

    print(f"OK Header field validation complete: {len(found_fields)} fields")

def main():
    print("Generating TransactionHeader from Go YAML sources...")

    # Ensure output directories exist
    GEN_DIR.mkdir(parents=True, exist_ok=True)

    # Load YAML file
    print("\nLoading YAML files...")
    transaction_yaml_path = GO_REPO / "protocol" / "transaction.yml"
    yaml_data = load_yaml_file(transaction_yaml_path)

    if not yaml_data:
        print("ERROR Failed to load transaction.yml")
        sys.exit(1)

    # Extract header fields
    print("\nExtracting TransactionHeader schema...")
    header_fields = extract_transaction_header(yaml_data)

    # Extract nested types
    print("\nExtracting nested types...")
    nested_types = extract_nested_types(yaml_data)

    # Validate fields
    print("\nValidating header fields...")
    validate_required_fields(header_fields)

    # Generate Rust code
    print("\nGenerating Rust code...")
    rust_code = generate_header_rs(header_fields, nested_types)

    # Write header.rs
    header_rs_path = GEN_DIR / "header.rs"
    with open(header_rs_path, 'w', encoding='utf-8') as f:
        f.write(rust_code)
    print(f"Generated: {header_rs_path}")

    # Generate and write manifest
    print("\nGenerating manifest...")
    manifest = generate_manifest(header_fields, nested_types)
    manifest_path = GEN_DIR / "header_manifest.json"
    with open(manifest_path, 'w', encoding='utf-8') as f:
        json.dump(manifest, f, indent=2)
    print(f"Generated: {manifest_path}")

    print(f"\nSuccessfully generated TransactionHeader!")
    print(f"   Fields: {len(header_fields)}")
    print(f"   Nested types: {len(nested_types)}")
    print(f"   Required fields: {sum(1 for f in header_fields if not f.optional)}")
    print(f"   Optional fields: {sum(1 for f in header_fields if f.optional)}")

    return 0

if __name__ == "__main__":
    sys.exit(main())