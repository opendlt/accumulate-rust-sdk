#!/usr/bin/env python3
"""
API Methods Code Generator for Rust Accumulate SDK

Generates src/generated/api_methods.rs from Go YAML sources.
Canonical truth: internal/api/v2/methods.yml

Requirements:
- Extract exactly 35 public RPC methods from YAML
- Generate Rust structs for params and results
- Create strongly-typed client with AccumulateRpc trait
- Enforce strict count gate (35 methods)
- Create manifest JSON with metadata
"""

import yaml
import json
import os
import sys
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Any, Optional, Tuple

# Constants
GO_REPO = Path(r"C:\Accumulate_Stuff\accumulate")
GO_METHODS = GO_REPO / "internal" / "api" / "v2" / "methods.yml"
RUST_ROOT = Path(r"C:\Accumulate_Stuff\opendlt-rust-v2v3-sdk\unified")
SRC_DIR = RUST_ROOT / "src"
GEN_DIR = SRC_DIR / "generated"

class ApiMethod:
    """Represents a parsed API method from YAML"""

    def __init__(self, name: str, method_def: Dict[str, Any]):
        self.name = name
        self.rpc_name = method_def.get('rpc', name.lower())
        self.description = method_def.get('description', '')
        self.input_type = method_def.get('input', None)
        self.output_type = method_def.get('output', None)
        self.call_params = method_def.get('call-params', [])

    def get_params_struct_name(self) -> str:
        """Generate Rust struct name for parameters"""
        return f"{self.name}Params"

    def get_result_struct_name(self) -> str:
        """Generate Rust struct name for result"""
        return f"{self.name}Response"

    def get_rust_method_name(self) -> str:
        """Generate Rust method name (snake_case)"""
        # Convert CamelCase to snake_case
        import re
        s1 = re.sub('([a-z0-9])([A-Z])', r'\1_\2', self.name)
        return s1.lower()

    def get_method_info(self) -> Dict[str, Any]:
        """Get method information for manifest"""
        return {
            'name': self.rpc_name,
            'params': self.get_params_struct_name(),
            'result': self.get_result_struct_name(),
            'description': self.description
        }

def load_yaml_file(file_path: Path) -> Dict[str, Any]:
    """Load a YAML file and return its contents"""
    if not file_path.exists():
        print(f"ERROR File not found: {file_path}")
        sys.exit(1)

    try:
        with open(file_path, 'r', encoding='utf-8') as f:
            content = yaml.safe_load(f)
            print(f"OK Loaded {file_path}")
            return content or {}
    except yaml.YAMLError as e:
        print(f"ERROR loading {file_path}: {e}")
        sys.exit(1)

def extract_api_methods(yaml_data: Dict[str, Any]) -> List[ApiMethod]:
    """Extract API methods from YAML data"""
    methods = []

    for method_name, method_def in yaml_data.items():
        if isinstance(method_def, dict) and 'rpc' in method_def:
            methods.append(ApiMethod(method_name, method_def))

    print(f"Found {len(methods)} API methods")

    # Enforce strict count gate
    if len(methods) != 35:
        print(f"ERROR Expected exactly 35 API methods, found {len(methods)}")
        print("Missing/Extra DIFF:")
        expected_count = 35
        actual_count = len(methods)
        if actual_count < expected_count:
            print(f"  Missing: {expected_count - actual_count} methods")
        else:
            print(f"  Extra: {actual_count - expected_count} methods")
        sys.exit(2)

    return methods

def map_yaml_type_to_rust(yaml_type: str) -> str:
    """Map YAML type names to Rust types"""
    # Common type mappings
    type_mapping = {
        'string': 'String',
        'uint64': 'u64',
        'uint32': 'u32',
        'int64': 'i64',
        'int32': 'i32',
        'bool': 'bool',
        'bytes': 'Vec<u8>',
        'hash': 'Vec<u8>',
        'url': 'String',
        # Query/Response types
        'GeneralQuery': 'GeneralQuery',
        'DirectoryQuery': 'DirectoryQuery',
        'TxnQuery': 'TxnQuery',
        'MetricsQuery': 'MetricsQuery',
        'StatusResponse': 'StatusResponse',
        'ChainQueryResponse': 'ChainQueryResponse',
        'TransactionQueryResponse': 'TransactionQueryResponse',
        'MultiResponse': 'MultiResponse',
        'TxResponse': 'TxResponse',
        'DescriptionResponse': 'DescriptionResponse',
        # Protocol types
        'protocol.AcmeFaucet': 'serde_json::Value',
        # Generic fallback
        'QueryOptions': 'QueryOptions',
        'QueryPagination': 'QueryPagination',
    }

    # Handle union types like "A|B|C"
    if '|' in yaml_type:
        return 'serde_json::Value'  # Use generic JSON for union types

    return type_mapping.get(yaml_type, 'serde_json::Value')

def generate_params_struct(method: ApiMethod) -> str:
    """Generate Rust struct for method parameters"""
    struct_name = method.get_params_struct_name()

    if not method.input_type:
        # No input parameters
        return f"""#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct {struct_name} {{
    // No parameters
}}"""

    # For now, use a simple generic approach since we don't have detailed field schemas
    # In a full implementation, you'd parse the input type structure
    rust_type = map_yaml_type_to_rust(method.input_type)

    if method.input_type == 'protocol.AcmeFaucet':
        # Special case for faucet - use the transaction body type
        return f"""#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct {struct_name} {{
    pub url: String,
}}"""
    elif 'Query' in method.input_type:
        # Query types typically have URL and options
        return f"""#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct {struct_name} {{
    pub url: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub options: Option<serde_json::Value>,
}}"""
    else:
        # Generic structure
        return f"""#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct {struct_name} {{
    #[serde(flatten)]
    pub params: serde_json::Value,
}}"""

def generate_result_struct(method: ApiMethod) -> str:
    """Generate Rust struct for method result"""
    struct_name = method.get_result_struct_name()

    if not method.output_type:
        # No output
        return f"""#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct {struct_name} {{
    // No result data
}}"""

    # Handle union types and specific response types
    if '|' in method.output_type:
        return f"""#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct {struct_name} {{
    #[serde(flatten)]
    pub data: serde_json::Value,
}}"""

    # Specific response type structures
    if method.output_type == 'StatusResponse':
        return f"""#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct {struct_name} {{
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub last_block_time: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub version: Option<String>,
}}"""
    elif method.output_type == 'TxResponse':
        return f"""#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct {struct_name} {{
    pub transaction_hash: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub simple_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub code: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub message: Option<String>,
}}"""
    else:
        # Generic response structure
        return f"""#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct {struct_name} {{
    #[serde(flatten)]
    pub data: serde_json::Value,
}}"""

def generate_client_methods(methods: List[ApiMethod]) -> str:
    """Generate client implementation with all methods"""
    method_impls = []

    for method in methods:
        method_name = method.get_rust_method_name()
        params_type = method.get_params_struct_name()
        result_type = method.get_result_struct_name()
        rpc_name = method.rpc_name

        method_impl = f"""    pub async fn {method_name}(&self, params: {params_type}) -> Result<{result_type}, Error> {{
        self.transport.rpc_call("{rpc_name}", &params).await
    }}"""
        method_impls.append(method_impl)

    methods_code = '\n\n'.join(method_impls)

    return f"""impl<C: AccumulateRpc + Send + Sync> AccumulateClient<C> {{
{methods_code}
}}"""

def generate_test_helpers(methods: List[ApiMethod]) -> str:
    """Generate test helper functions for minimal params/results"""
    helpers = []

    for method in methods:
        method_name = method.rpc_name
        params_type = method.get_params_struct_name()
        result_type = method.get_result_struct_name()

        # Generate minimal params
        if not method.input_type:
            params_json = 'json!({})'
        elif method.input_type == 'protocol.AcmeFaucet':
            params_json = 'json!({"url": "acc://test.acme"})'
        elif 'Query' in method.input_type:
            params_json = 'json!({"url": "acc://test.acme"})'
        else:
            params_json = 'json!({})'

        # Generate minimal result
        if method.output_type == 'StatusResponse':
            result_json = 'json!({"ok": true})'
        elif method.output_type == 'TxResponse':
            result_json = 'json!({"transactionHash": "deadbeef"})'
        else:
            result_json = 'json!({"data": {}})'

        helper = f'        "{method_name}" => Some(({params_json}, {result_json})),'
        helpers.append(helper)

    helpers_code = '\n'.join(helpers)

    return f"""#[cfg(test)]
pub fn __minimal_pair_for_test(method_name: &str) -> Option<(serde_json::Value, serde_json::Value)> {{
    use serde_json::json;
    match method_name {{
{helpers_code}
        _ => None,
    }}
}}"""

def generate_api_methods_rs(methods: List[ApiMethod]) -> str:
    """Generate the complete api_methods.rs file"""
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    # Generate structs for each method
    param_structs = []
    result_structs = []

    for method in methods:
        param_structs.append(generate_params_struct(method))
        result_structs.append(generate_result_struct(method))

    params_code = '\n\n'.join(param_structs)
    results_code = '\n\n'.join(result_structs)
    client_code = generate_client_methods(methods)
    test_helpers = generate_test_helpers(methods)

    return f"""//! GENERATED FILE - DO NOT EDIT
//! Source: internal/api/v2/methods.yml
//! Generated: {timestamp}

use serde::{{Serialize, Deserialize}};
use crate::errors::Error;
use crate::generated::header::TransactionHeader;
use crate::generated::transactions::TransactionBody;
use async_trait::async_trait;

// AccumulateRpc trait for transport abstraction
#[async_trait]
pub trait AccumulateRpc {{
    async fn rpc_call<TParams: Serialize + Send + Sync, TResult: for<'de> Deserialize<'de>>(
        &self, method: &str, params: &TParams
    ) -> Result<TResult, Error>;
}}

// Generic client wrapper
pub struct AccumulateClient<C> {{
    pub transport: C,
}}

impl<C> AccumulateClient<C> {{
    pub fn new(transport: C) -> Self {{
        Self {{ transport }}
    }}
}}

// Parameter structures
{params_code}

// Result structures
{results_code}

// Client implementation with strongly-typed methods
{client_code}

{test_helpers}"""

def generate_manifest(methods: List[ApiMethod]) -> Dict[str, Any]:
    """Generate the API manifest JSON"""
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S")

    method_info = []
    for method in methods:
        method_info.append(method.get_method_info())

    return {
        'generated_at': timestamp,
        'methods': method_info,
        'counts': {'api': len(methods)}
    }

def main():
    print("Generating API methods from Go YAML sources...")

    # Ensure output directories exist
    GEN_DIR.mkdir(parents=True, exist_ok=True)

    print("\nLoading YAML files...")
    yaml_data = load_yaml_file(GO_METHODS)

    print("\nExtracting API methods...")
    methods = extract_api_methods(yaml_data)

    # List methods for verification
    print(f"\nFound {len(methods)} API methods:")
    for i, method in enumerate(methods, 1):
        print(f"  {i:2d}: {method.name} (rpc: {method.rpc_name})")

    print("\nGenerating Rust code...")
    api_code = generate_api_methods_rs(methods)

    # Write api_methods.rs
    api_file = GEN_DIR / "api_methods.rs"
    with open(api_file, 'w', encoding='utf-8') as f:
        f.write(api_code)
    print(f"Generated: {api_file}")

    print("\nGenerating manifest...")
    manifest = generate_manifest(methods)

    # Write manifest
    manifest_file = GEN_DIR / "api_manifest.json"
    with open(manifest_file, 'w', encoding='utf-8') as f:
        json.dump(manifest, f, indent=2)
    print(f"Generated: {manifest_file}")

    print(f"\nSuccessfully generated API methods!")
    print(f"   Methods: {len(methods)}")
    print(f"   Strictness: {'PASS' if len(methods) == 35 else 'FAIL'}")

    return 0

if __name__ == "__main__":
    sys.exit(main())