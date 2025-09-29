"""
Cryptographic utilities for Accumulate blockchain operations.

This module provides hashing and signature utilities that use canonical JSON
to ensure compatibility with Dart/TypeScript SDK implementations.
"""

import hashlib
from typing import Any, Union

try:
    from .canonjson import canonicalize_for_hashing, dumps_canonical
except ImportError:
    # For standalone testing
    from canonjson import canonicalize_for_hashing, dumps_canonical


def hash_json(obj: Any) -> bytes:
    """
    Hash a JSON-serializable object using canonical JSON and SHA-256.

    This function ensures that identical objects produce identical hashes
    across different language implementations by using canonical JSON.

    Args:
        obj: JSON-serializable object to hash

    Returns:
        32-byte SHA-256 hash

    Example:
        >>> tx = {"header": {"principal": "acc://alice.acme"}, "body": {...}}
        >>> tx_hash = hash_json(tx)
        >>> len(tx_hash)
        32
    """
    canonical_bytes = canonicalize_for_hashing(obj)
    return hashlib.sha256(canonical_bytes).digest()


def hash_json_hex(obj: Any) -> str:
    """
    Hash a JSON-serializable object and return hex string.

    Convenience function that combines hash_json with hex encoding.

    Args:
        obj: JSON-serializable object to hash

    Returns:
        64-character hex string of SHA-256 hash

    Example:
        >>> tx = {"header": {"principal": "acc://alice.acme"}, "body": {...}}
        >>> tx_hash_hex = hash_json_hex(tx)
        >>> len(tx_hash_hex)
        64
    """
    return hash_json(obj).hex()


def hash_transaction_for_signing(transaction: dict) -> bytes:
    """
    Hash a transaction object for signing purposes.

    This follows the standard pattern where signatures are computed over
    the hash of the canonical JSON representation of the transaction.

    Args:
        transaction: Transaction dict with 'header' and 'body' fields

    Returns:
        32-byte SHA-256 hash suitable for signing

    Example:
        >>> tx = {
        ...     "header": {"principal": "acc://alice.acme/tokens", "timestamp": 1234567890123},
        ...     "body": {"type": "send-tokens", "to": [{"url": "acc://bob.acme/tokens", "amount": "1000"}]}
        ... }
        >>> signing_hash = hash_transaction_for_signing(tx)
        >>> len(signing_hash)
        32
    """
    return hash_json(transaction)


def verify_canonical_hash(obj: Any, expected_hash: Union[str, bytes]) -> bool:
    """
    Verify that an object's canonical hash matches an expected value.

    This is useful for validating objects against known hash values
    from test vectors or other implementations.

    Args:
        obj: Object to hash
        expected_hash: Expected hash as hex string or bytes

    Returns:
        True if hashes match, False otherwise

    Example:
        >>> tx = {"header": {"timestamp": 123}, "body": {"type": "test"}}
        >>> expected = "abc123..."  # Known good hash
        >>> verify_canonical_hash(tx, expected)
        True
    """
    computed_hash = hash_json(obj)

    if isinstance(expected_hash, str):
        expected_bytes = bytes.fromhex(expected_hash)
    else:
        expected_bytes = expected_hash

    return computed_hash == expected_bytes


def canonical_json_string(obj: Any) -> str:
    """
    Get canonical JSON string representation of an object.

    This is a convenience wrapper around dumps_canonical for use in
    crypto contexts where the string representation is needed.

    Args:
        obj: Object to serialize

    Returns:
        Canonical JSON string

    Example:
        >>> tx = {"z": 3, "a": 1}
        >>> canonical_json_string(tx)
        '{"a":1,"z":3}'
    """
    return dumps_canonical(obj)


# Convenience constants and utilities

def sha256_bytes(data: bytes) -> bytes:
    """SHA-256 hash of raw bytes."""
    return hashlib.sha256(data).digest()


def sha256_hex(data: bytes) -> str:
    """SHA-256 hash of raw bytes as hex string."""
    return hashlib.sha256(data).hexdigest()


def double_sha256(data: bytes) -> bytes:
    """Double SHA-256 hash (hash of hash)."""
    return sha256_bytes(sha256_bytes(data))


class TransactionHasher:
    """
    Utility class for computing transaction hashes in a consistent way.

    This class ensures that transaction hashing follows the same patterns
    as the Dart/TypeScript SDKs.
    """

    @staticmethod
    def hash_for_signing(transaction: dict) -> bytes:
        """Hash transaction for signature creation/verification."""
        return hash_transaction_for_signing(transaction)

    @staticmethod
    def hash_for_signing_hex(transaction: dict) -> str:
        """Hash transaction for signing as hex string."""
        return TransactionHasher.hash_for_signing(transaction).hex()

    @staticmethod
    def verify_hash(transaction: dict, expected_hash: Union[str, bytes]) -> bool:
        """Verify transaction hash matches expected value."""
        return verify_canonical_hash(transaction, expected_hash)

    @staticmethod
    def get_canonical_json(transaction: dict) -> str:
        """Get canonical JSON representation of transaction."""
        return canonical_json_string(transaction)


if __name__ == '__main__':
    # Test with known good transaction
    test_transaction = {
        "header": {
            "principal": "acc://alice.acme/tokens",
            "timestamp": 1234567890123
        },
        "body": {
            "type": "send-tokens",
            "to": [{
                "url": "acc://bob.acme/tokens",
                "amount": "1000"
            }]
        }
    }

    # This should match the expected hash from the golden fixtures
    expected_hash = "4be49c59c717f1984646998cecac0e5225378d9bbe2e18928272a85b7dfcb608"

    computed_hash = hash_json_hex(test_transaction)
    canonical = canonical_json_string(test_transaction)

    print(f"Canonical JSON: {canonical}")
    print(f"Computed hash:  {computed_hash}")
    print(f"Expected hash:  {expected_hash}")
    print(f"Hashes match:   {computed_hash == expected_hash}")

    # Verify using the utility function
    print(f"Verify function: {verify_canonical_hash(test_transaction, expected_hash)}")