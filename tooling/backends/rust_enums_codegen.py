#!/usr/bin/env python3
"""
Rust Enums Code Generator for Accumulate Protocol
Reads Go YAML truth and generates Rust enums with exact wire compatibility.
"""

import os
import sys
import yaml
import json
from datetime import datetime
from pathlib import Path

# Paths
GO_REPO = r"C:\Accumulate_Stuff\accumulate"
RUST_ROOT = r"C:\Accumulate_Stuff\opendlt-rust-v2v3-sdk\unified"
SRC_DIR = os.path.join(RUST_ROOT, "src")
GEN_DIR = os.path.join(SRC_DIR, "generated")

EXPECTED_ENUM_COUNT = 14

def load_yaml_with_anchors(file_path):
    """Load YAML file with anchor resolution."""
    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            return yaml.safe_load(f)
    except Exception as e:
        print(f"Error loading {file_path}: {e}")
        return {}

def pascal_case(snake_str):
    """Convert snake_case to PascalCase."""
    return ''.join(word.capitalize() for word in snake_str.split('_'))

def extract_wire_tag(variant_data):
    """Extract the wire tag from variant data, preferring label > aliases > lowercase name."""
    if isinstance(variant_data, dict):
        # Check for explicit label first
        if "label" in variant_data:
            return variant_data["label"]

        # Check for aliases
        if "aliases" in variant_data and variant_data["aliases"]:
            return variant_data["aliases"][0]

    # If no explicit wire format, return None to use variant name
    return None

def normalize_variant_name(name):
    """Normalize variant names to valid Rust identifiers."""
    # Handle special cases
    name_map = {
        "V1SignatureAnchoring": "V1SignatureAnchoring",
        "V1DoubleHashEntries": "V1DoubleHashEntries",
        "V1Halt": "V1Halt",
        "V2Baikonur": "V2Baikonur",
        "V2Vandenberg": "V2Vandenberg",
        "V2Jiuquan": "V2Jiuquan",
    }

    if name in name_map:
        return name_map[name]

    return pascal_case(name) if '_' in name else name

def generate_enum_rust_code(enum_name, variants_data):
    """Generate Rust enum code for a single enum."""
    variants = []

    # Sort variants by value to maintain stable order
    sorted_variants = sorted(variants_data.items(),
                           key=lambda x: x[1].get('value', 0) if isinstance(x[1], dict) else 0)

    for variant_name, variant_data in sorted_variants:
        rust_variant = normalize_variant_name(variant_name)
        wire_tag = extract_wire_tag(variant_data)

        if wire_tag:
            # Use explicit wire tag
            variants.append(f'    #[serde(rename = "{wire_tag}")]\n    {rust_variant},')
        else:
            # Use variant name as wire tag (convert to appropriate case)
            if enum_name == "TransactionType":
                # Transaction types use camelCase
                wire_tag = variant_name[0].lower() + variant_name[1:] if len(variant_name) > 1 else variant_name.lower()
                variants.append(f'    #[serde(rename = "{wire_tag}")]\n    {rust_variant},')
            else:
                # Most other enums use the variant name directly or lowercase
                wire_tag = variant_name.lower()
                variants.append(f'    #[serde(rename = "{wire_tag}")]\n    {rust_variant},')

    # Generate the enum
    enum_code = f'''#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum {enum_name} {{
{chr(10).join(variants)}
}}'''

    return enum_code, [v.split(',')[0].strip().split()[-1] for v in variants]

def generate_test_helper(all_enums_data):
    """Generate test helper functions for roundtrip testing."""
    helper_code = '''
pub fn __roundtrip_one(enum_name: &str, tag: &str) -> Result<(), Box<dyn std::error::Error>> {
    match enum_name {'''

    for enum_name, variants in all_enums_data.items():
        helper_code += f'''
        "{enum_name}" => {{
            let v = serde_json::Value::String(tag.to_string());
            let val: {enum_name} = serde_json::from_value(v.clone())?;
            let back = serde_json::to_value(&val)?;
            if back != v {{
                return Err(format!("Roundtrip failed for {{}}::{{}}: expected {{}}, got {{}}",
                    enum_name, tag, v, back).into());
            }}
        }}'''

    helper_code += '''
        _ => return Err(format!("Unknown enum: {}", enum_name).into()),
    }
    Ok(())
}

pub fn __get_all_enum_variants() -> std::collections::HashMap<String, Vec<String>> {
    let mut map = std::collections::HashMap::new();'''

    for enum_name, variants in all_enums_data.items():
        variant_list = ', '.join(f'"{v}".to_string()' for v in variants)
        helper_code += f'''
    map.insert("{enum_name}".to_string(), vec![{variant_list}]);'''

    helper_code += '''
    map
}'''

    return helper_code

def main():
    print("=== Rust Enums Code Generator ===")

    # Load Go truth YAML
    enums_file = os.path.join(GO_REPO, "protocol", "enums.yml")
    if not os.path.exists(enums_file):
        print(f"ERROR: {enums_file} not found!")
        return 1

    print(f"Loading: {enums_file}")
    enums_data = load_yaml_with_anchors(enums_file)

    if not enums_data:
        print("ERROR: Failed to load enum data!")
        return 1

    # Validate enum count
    enum_count = len(enums_data)
    print(f"Found {enum_count} enums")

    if enum_count != EXPECTED_ENUM_COUNT:
        print(f"ENUM_COUNT_FAIL: found {enum_count}, expected {EXPECTED_ENUM_COUNT}")

        expected_enums = {
            "ExecutorVersion", "PartitionType", "DataEntryType", "ObjectType",
            "SignatureType", "KeyPageOperationType", "AccountAuthOperationType",
            "NetworkMaintenanceOperationType", "TransactionMax", "TransactionType",
            "AccountType", "AllowedTransactionBit", "VoteType", "BookType"
        }
        found_enums = set(enums_data.keys())

        missing = expected_enums - found_enums
        extra = found_enums - expected_enums

        if missing:
            print(f"Missing: {sorted(missing)}")
        if extra:
            print(f"Extra: {sorted(extra)}")

        return 2

    # Generate Rust code
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    rust_code = f'''// GENERATED FILE - DO NOT EDIT
// Source: protocol/enums.yml | Generated: {timestamp}

use serde::{{Serialize, Deserialize}};

'''

    all_enums_data = {}
    enum_manifest = {
        "generated_at": timestamp,
        "enums": [],
        "counts": {"enums": enum_count}
    }

    # Sort enums by name for stable output
    for enum_name in sorted(enums_data.keys()):
        enum_data = enums_data[enum_name]
        print(f"Generating enum: {enum_name}")

        enum_code, variant_names = generate_enum_rust_code(enum_name, enum_data)
        rust_code += enum_code + "\n\n"

        # Collect wire tags for manifest
        wire_tags = []
        sorted_variants = sorted(enum_data.items(),
                               key=lambda x: x[1].get('value', 0) if isinstance(x[1], dict) else 0)

        for variant_name, variant_data in sorted_variants:
            wire_tag = extract_wire_tag(variant_data)
            if not wire_tag:
                if enum_name == "TransactionType":
                    wire_tag = variant_name[0].lower() + variant_name[1:] if len(variant_name) > 1 else variant_name.lower()
                else:
                    wire_tag = variant_name.lower()
            wire_tags.append(wire_tag)

        all_enums_data[enum_name] = wire_tags
        enum_manifest["enums"].append({
            "name": enum_name,
            "variants": wire_tags
        })

    # Add test helpers
    rust_code += generate_test_helper(all_enums_data)

    # Ensure output directory exists
    os.makedirs(GEN_DIR, exist_ok=True)

    # Write Rust file
    rust_file = os.path.join(GEN_DIR, "enums.rs")
    with open(rust_file, 'w', encoding='utf-8') as f:
        f.write(rust_code)

    print(f"Generated: {rust_file}")

    # Write manifest
    manifest_file = os.path.join(GEN_DIR, "enums_manifest.json")
    with open(manifest_file, 'w', encoding='utf-8') as f:
        json.dump(enum_manifest, f, indent=2)

    print(f"Generated: {manifest_file}")

    print(f"SUCCESS: Generated {enum_count} enums with exact wire compatibility")
    return 0

if __name__ == "__main__":
    sys.exit(main())