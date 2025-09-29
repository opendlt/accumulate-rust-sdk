"""
Accumulate Client Library for Python

This package provides utilities for working with the Accumulate blockchain,
including canonical JSON encoding that matches the Dart/TypeScript SDK exactly.
"""

from .canonjson import dumps_canonical, canonicalize_for_hashing
from .crypto import (
    hash_json, hash_json_hex, hash_transaction_for_signing,
    verify_canonical_hash, canonical_json_string, TransactionHasher
)

__all__ = [
    'dumps_canonical', 'canonicalize_for_hashing',
    'hash_json', 'hash_json_hex', 'hash_transaction_for_signing',
    'verify_canonical_hash', 'canonical_json_string', 'TransactionHasher'
]