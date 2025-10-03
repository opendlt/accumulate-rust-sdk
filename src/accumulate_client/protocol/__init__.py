"""
Accumulate protocol implementation matching TypeScript SDK.

This module provides transaction hashing and protocol utilities
that match the TypeScript SDK exactly.
"""

from .hashing import (
    hash_transaction, hash_transaction_hex, verify_transaction_hash,
    hash_for_signing, hash_for_ed25519_signing, hash_transaction_header,
    hash_transaction_body, create_signature_metadata_hash
)

__all__ = [
    'hash_transaction', 'hash_transaction_hex', 'verify_transaction_hash',
    'hash_for_signing', 'hash_for_ed25519_signing', 'hash_transaction_header',
    'hash_transaction_body', 'create_signature_metadata_hash'
]