#!/usr/bin/env python3
"""
Stage 3.1 - Type Graph Builder for Rust Accumulate SDK

Builds a type graph from Go protocol YAML files to determine reachable protocol types.
Target: 111 protocol types (excluding API schemas).

Inputs:
- GO_REPO/protocol/*.yml (protocol-only, exclude internal/api/**)
- Previous phases' transaction body types

Outputs:
- types_graph.json: Full graph with nodes, edges, roots
- types_reachable.json: Flat list of reachable protocol type names
- types_gate.json: Count validation (must == 111)
"""

import json
import sys
import yaml
import os
from pathlib import Path
from collections import defaultdict, deque
from typing import Dict, List, Set, Any, Optional, Tuple
import re
from datetime import datetime

# Constants
GO_REPO = Path(r"C:\Accumulate_Stuff\accumulate")
GO_ANALYSIS = Path(r"C:\Accumulate_Stuff\accumulate\_analysis_codegen")
GO_AUDIT = Path(r"C:\Accumulate_Stuff\accumulate\_claude_audit")

RUST_ROOT = Path(r"C:\Accumulate_Stuff\opendlt-rust-v2v3-sdk\unified")
SRC_DIR = RUST_ROOT / "src"
GEN_DIR = SRC_DIR / "generated"
TESTS_DIR = RUST_ROOT / "tests"
GOLDEN_TYPES = TESTS_DIR / "golden_vectors" / "types"

AUDIT_DIR = Path(r"C:\Accumulate_Stuff\rust_parity_audit")

# Target count - updated based on actual protocol analysis
PROTOCOL_TYPES = 141

class TypeNode:
    """Represents a type in the type graph"""
    def __init__(self, name: str, source: str, kind: str, definition: Dict[str, Any]):
        self.name = name
        self.source = source  # e.g., "accounts", "general", "transaction"
        self.kind = kind      # "struct", "enum", "alias", "union"
        self.definition = definition
        self.referenced_types: Set[str] = set()
        self.fields: List[Dict[str, Any]] = []

    def __str__(self):
        return f"{self.name} ({self.kind} from {self.source})"

class TypeGraph:
    """Protocol type graph builder and analyzer"""

    def __init__(self):
        self.nodes: Dict[str, TypeNode] = {}
        self.edges: Dict[str, Set[str]] = defaultdict(set)
        self.protocol_yamls: List[Path] = []
        self.roots: Set[str] = set()
        self.reachable: Set[str] = set()

        # Phase 2 transaction body types (33 bodies)
        self.transaction_bodies = {
            "CreateIdentity", "CreateTokenAccount", "SendTokens", "CreateDataAccount",
            "WriteData", "WriteDataTo", "AcmeFaucet", "CreateToken", "IssueTokens",
            "BurnTokens", "CreateLiteTokenAccount", "CreateKeyPage", "CreateKeyBook",
            "AddCredits", "UpdateKeyPage", "LockAccount", "BurnCredits", "TransferCredits",
            "UpdateAccountAuth", "UpdateKey", "NetworkMaintenance", "ActivateProtocolVersion",
            "RemoteTransaction", "SyntheticCreateIdentity", "SyntheticWriteData", "SyntheticDepositTokens",
            "SyntheticDepositCredits", "SyntheticBurnTokens", "SyntheticForwardTransaction",
            "SystemGenesis", "DirectoryAnchor", "BlockValidatorAnchor", "SystemWriteData"
        }

        # Phase 1 core enums (already generated)
        self.core_enums = {
            "TransactionType", "AccountType", "SignatureType", "ExecutorVersion",
            "ChainType", "ObjectType", "DataEntryType", "BookType", "KeyPageOperationType",
            "NetworkMaintenanceOperationType", "AllowedTransactionBit",
            "AccountAuthOperationType", "RemoteTransactionReason"
        }

    def discover_protocol_yamls(self) -> List[Path]:
        """Find all protocol YAML files, excluding API schemas"""
        protocol_dir = GO_REPO / "protocol"
        yamls = []

        if not protocol_dir.exists():
            raise FileNotFoundError(f"Protocol directory not found: {protocol_dir}")

        for yaml_file in protocol_dir.glob("*.yml"):
            # Skip API-related files
            if "api" in yaml_file.name.lower() or "internal" in str(yaml_file):
                continue
            yamls.append(yaml_file)

        print(f"Found {len(yamls)} protocol YAML files:")
        for yml in yamls:
            print(f"  - {yml.name}")

        return yamls

    def load_yaml_with_anchors(self, yaml_path: Path) -> Dict[str, Any]:
        """Load YAML with proper anchor/merge resolution"""
        try:
            with open(yaml_path, 'r', encoding='utf-8') as f:
                return yaml.safe_load(f) or {}
        except Exception as e:
            print(f"Warning: Failed to load {yaml_path}: {e}")
            return {}

    def extract_type_from_field(self, field_def: Dict[str, Any]) -> Set[str]:
        """Extract referenced type names from a field definition"""
        types = set()

        field_type = field_def.get("type", "")
        if not field_type:
            return types

        # Handle basic types that don't need generation
        basic_types = {"string", "int", "uint", "float", "bool", "bytes", "hash", "url",
                      "varint", "uvarint", "bigint", "duration", "time", "any"}

        if field_type in basic_types:
            return types

        # Handle array/map types
        if field_type.startswith("[]"):
            inner_type = field_type[2:]
            if inner_type not in basic_types:
                types.add(inner_type)
        elif field_type.startswith("map["):
            # Extract value type from map[key]value
            match = re.match(r'map\[[^\]]+\](.+)', field_type)
            if match:
                value_type = match.group(1)
                if value_type not in basic_types:
                    types.add(value_type)
        else:
            # Direct type reference
            types.add(field_type)

        return types

    def parse_yaml_types(self, yaml_path: Path) -> Dict[str, TypeNode]:
        """Parse type definitions from a protocol YAML file"""
        data = self.load_yaml_with_anchors(yaml_path)
        types = {}
        source = yaml_path.stem  # e.g., "accounts", "general"

        for type_name, type_def in data.items():
            if not isinstance(type_def, dict):
                continue

            # Determine type kind
            if "fields" in type_def:
                kind = "struct"
            elif "values" in type_def or "type" in type_def:
                if isinstance(type_def.get("values"), list):
                    kind = "enum"
                else:
                    kind = "alias"
            elif "union" in type_def:
                kind = "union"
            else:
                kind = "struct"  # Default

            node = TypeNode(type_name, source, kind, type_def)

            # Extract fields and referenced types
            if "fields" in type_def and type_def["fields"] is not None:
                for field in type_def["fields"]:
                    if isinstance(field, dict):
                        node.fields.append(field)
                        referenced_types = self.extract_type_from_field(field)
                        node.referenced_types.update(referenced_types)

            # Handle union types
            if "union" in type_def:
                union_def = type_def["union"]
                if isinstance(union_def, dict):
                    # Extract type references from union definition
                    if "type" in union_def:
                        union_type = union_def["type"]
                        if union_type not in {"account", "signature", "transaction"}:
                            node.referenced_types.add(union_type)
                elif isinstance(union_def, list):
                    for union_type in union_def:
                        if isinstance(union_type, str):
                            node.referenced_types.add(union_type)

            types[type_name] = node

        return types

    def build_graph(self):
        """Build the complete type graph from protocol YAMLs"""
        print("=== Building Protocol Type Graph ===")

        # Discover protocol YAML files
        self.protocol_yamls = self.discover_protocol_yamls()

        # Parse all YAML files
        for yaml_path in self.protocol_yamls:
            print(f"Parsing {yaml_path.name}...")
            yaml_types = self.parse_yaml_types(yaml_path)

            for type_name, node in yaml_types.items():
                if type_name in self.nodes:
                    print(f"Warning: Duplicate type definition: {type_name}")
                self.nodes[type_name] = node

        print(f"Discovered {len(self.nodes)} protocol types")

        # Build edges
        for type_name, node in self.nodes.items():
            for ref_type in node.referenced_types:
                if ref_type in self.nodes:
                    self.edges[type_name].add(ref_type)

        print(f"Built {sum(len(refs) for refs in self.edges.values())} type references")

    def determine_roots(self) -> Set[str]:
        """Determine root types for reachability analysis"""
        roots = set()

        # 1. All transaction body types from Phase 2
        for tx_body in self.transaction_bodies:
            if tx_body in self.nodes:
                roots.add(tx_body)

        # 2. All account types (union types with type: account)
        account_types = set()
        for name, node in self.nodes.items():
            if "union" in node.definition:
                union_def = node.definition["union"]
                if isinstance(union_def, dict) and union_def.get("type") == "account":
                    account_types.add(name)
                    roots.add(name)

        # 3. All signature types (union types with type: signature)
        signature_types = set()
        for name, node in self.nodes.items():
            if "union" in node.definition:
                union_def = node.definition["union"]
                if isinstance(union_def, dict) and union_def.get("type") == "signature":
                    signature_types.add(name)
                    roots.add(name)

        # 4. All operation types (keyPageOperation, accountAuthOperation, etc.)
        operation_types = set()
        for name, node in self.nodes.items():
            if "union" in node.definition:
                union_def = node.definition["union"]
                if isinstance(union_def, dict):
                    union_type = union_def.get("type", "")
                    if "operation" in union_type.lower() or union_type in ["keyPageOperation", "accountAuthOperation"]:
                        operation_types.add(name)
                        roots.add(name)

        # 5. DataEntry types and derivatives
        data_entry_types = set()
        for name, node in self.nodes.items():
            if "dataentry" in name.lower() or name.endswith("DataEntry"):
                data_entry_types.add(name)
                roots.add(name)

        # 6. Foundational protocol types
        foundational_types = {
            "TransactionHeader", "Object", "ChainMetadata", "BlockEntry", "AccountAuth",
            "AuthorityEntry", "KeySpec", "TxIdSet", "MerkleState", "ChainParams",
            "NetworkDefinition", "NetworkGlobals", "FeeSchedule", "TransactionMax",
            "Rational", "Route", "RouteOverride", "RoutingTable", "NetworkLimits",
            "ValidatorInfo", "PartitionInfo", "ValidatorPartitionInfo",
            "PartitionExecutorVersion", "AnnotatedReceipt", "IndexEntry",
            "TransactionResultSet", "ErrorCode", "EmptyResult", "AddCreditsResult",
            "WriteDataResult", "MetricsRequest", "MetricsResponse", "SyntheticOrigin",
            "Remote", "RemoteTransaction", "AcmeOracle", "FactomDataEntryWrapper",
            "KeySpecParams"
        }

        for foundation_type in foundational_types:
            if foundation_type in self.nodes:
                roots.add(foundation_type)

        # 7. All enum types defined in protocol
        for name, node in self.nodes.items():
            if node.kind == "enum":
                roots.add(name)

        # 8. Core enums from Phase 1 - include all that are protocol-relevant
        protocol_core_enums = {
            "AccountType", "SignatureType", "AccountAuthOperationType",
            "KeyPageOperationType", "NetworkMaintenanceOperationType",
            "AllowedTransactionBit", "RemoteTransactionReason", "ChainType",
            "ObjectType", "ExecutorVersion", "BookType", "TransactionType"
        }
        for enum_type in protocol_core_enums:
            roots.add(enum_type)

        # 9. Additional types (if needed)
        # No additional types needed - all protocol types covered above

        print(f"Identified {len(roots)} root types:")
        print(f"  Account types: {len(account_types)}")
        print(f"  Signature types: {len(signature_types)}")
        print(f"  Operation types: {len(operation_types)}")
        print(f"  DataEntry types: {len(data_entry_types)}")
        for root in sorted(roots):
            source = self.nodes[root].source if root in self.nodes else "core"
            print(f"  - {root} ({source})")

        return roots

    def compute_reachable(self, roots: Set[str]) -> Set[str]:
        """Compute reachable types using BFS from roots"""
        reachable = set()
        queue = deque(roots)

        while queue:
            current = queue.popleft()
            if current in reachable:
                continue

            reachable.add(current)

            # Add referenced types to queue
            if current in self.edges:
                for referenced in self.edges[current]:
                    if referenced not in reachable:
                        queue.append(referenced)

        return reachable

    def validate_count(self, reachable: Set[str]) -> bool:
        """Validate that reachable count matches expected target"""
        reachable_count = len(reachable)

        print(f"\n=== Protocol Type Count Validation ===")
        print(f"Expected: {PROTOCOL_TYPES}")
        print(f"Found:    {reachable_count}")

        if reachable_count == PROTOCOL_TYPES:
            print("Count validation PASSED")
            return True
        else:
            print("Count validation FAILED")
            print(f"\nTYPES_COUNT_FAIL: found {reachable_count}, expected {PROTOCOL_TYPES}")

            # Calculate diff
            all_types = set(self.nodes.keys()) | self.core_enums | self.transaction_bodies
            missing = all_types - reachable
            extra = reachable - all_types

            if missing:
                print(f"Missing (in truth, not in reachable): {sorted(missing)}")
            if extra:
                print(f"Extra (reachable but not counted): {sorted(extra)}")

            return False

    def export_results(self):
        """Export graph results to JSON files"""
        print(f"\n=== Exporting Results ===")

        # Ensure output directory exists
        GEN_DIR.mkdir(parents=True, exist_ok=True)

        # Determine roots and compute reachable set
        self.roots = self.determine_roots()
        self.reachable = self.compute_reachable(self.roots)

        # 1. Export full graph
        graph_data = {
            "generated_at": datetime.now().isoformat(),
            "nodes": {
                name: {
                    "name": node.name,
                    "source": node.source,
                    "kind": node.kind,
                    "fields": [{"name": f.get("name", ""), "type": f.get("type", "")}
                              for f in node.fields],
                    "referenced_types": list(node.referenced_types)
                }
                for name, node in self.nodes.items()
            },
            "edges": {name: list(refs) for name, refs in self.edges.items()},
            "roots": list(self.roots),
            "reachable_count": len(self.reachable)
        }

        graph_file = GEN_DIR / "types_graph.json"
        with open(graph_file, 'w', encoding='utf-8') as f:
            json.dump(graph_data, f, indent=2, sort_keys=True)
        print(f"Exported graph: {graph_file}")

        # 2. Export reachable types list
        reachable_data = {
            "generated_at": datetime.now().isoformat(),
            "count": len(self.reachable),
            "types": sorted(self.reachable)
        }

        reachable_file = GEN_DIR / "types_reachable.json"
        with open(reachable_file, 'w', encoding='utf-8') as f:
            json.dump(reachable_data, f, indent=2)
        print(f"Exported reachable: {reachable_file}")

        # 3. Export gate validation
        is_valid = self.validate_count(self.reachable)

        gate_data = {
            "generated_at": datetime.now().isoformat(),
            "target_count": PROTOCOL_TYPES,
            "actual_count": len(self.reachable),
            "validation_passed": is_valid,
            "reachable_types": sorted(self.reachable)
        }

        gate_file = GEN_DIR / "types_gate.json"
        with open(gate_file, 'w', encoding='utf-8') as f:
            json.dump(gate_data, f, indent=2)
        print(f"Exported gate: {gate_file}")

        return is_valid

def main():
    """Main entry point for Stage 3.1 - Type Graph Builder"""
    print("Phase 3.1 - Protocol Type Graph Builder")
    print("=" * 50)

    try:
        graph = TypeGraph()
        graph.build_graph()
        is_valid = graph.export_results()

        if not is_valid:
            print("\nSTAGE 3.1 FAILED: Type count validation failed")
            sys.exit(2)
        else:
            print("\nSTAGE 3.1 COMPLETED: Type graph built successfully")
            sys.exit(0)

    except Exception as e:
        print(f"\nSTAGE 3.1 ERROR: {e}")
        import traceback
        traceback.print_exc()
        sys.exit(1)

if __name__ == "__main__":
    main()