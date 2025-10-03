"""
Accumulate Client Library for Python

This package provides utilities for working with the Accumulate blockchain,
including canonical JSON encoding that matches the Dart/TypeScript SDK exactly.
"""

from .canonjson import dumps_canonical, canonicalize_for_hashing
from .crypto import Ed25519KeyPair, Ed25519Signature
from .protocol import (
    hash_transaction, hash_transaction_hex, verify_transaction_hash,
    hash_for_signing, hash_for_ed25519_signing
)

__all__ = [
    'dumps_canonical', 'canonicalize_for_hashing',
    'Ed25519KeyPair', 'Ed25519Signature',
    'hash_transaction', 'hash_transaction_hex', 'verify_transaction_hash',
    'hash_for_signing', 'hash_for_ed25519_signing'
]