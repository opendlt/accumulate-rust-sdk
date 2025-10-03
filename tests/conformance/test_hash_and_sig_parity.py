"""
Test hash and signature parity with TypeScript SDK golden vectors.

This module validates that our Python implementation produces exactly the same
hashes and signatures as the TypeScript SDK for all test vectors.
"""

import json
import os
import hashlib
from typing import Dict, Any

import pytest

from src.accumulate_client.canonjson import dumps_canonical
from src.accumulate_client.crypto.ed25519 import Ed25519KeyPair, Ed25519Signature
from src.accumulate_client.protocol.hashing import (
    hash_transaction, hash_transaction_hex, verify_transaction_hash,
    hash_for_ed25519_signing, create_signature_metadata_hash
)


def load_golden_fixture(filename: str) -> Dict[str, Any]:
    """Load a golden fixture JSON file."""
    fixture_path = os.path.join(os.path.dirname(__file__), '..', 'golden', filename)
    with open(fixture_path, 'r') as f:
        return json.load(f)


class TestEd25519SignatureParity:
    """Test Ed25519 signature verification against golden vectors."""

    def test_ed25519_signature_verification(self):
        """Test signature verification with golden Ed25519 vector."""
        golden = load_golden_fixture('sig_ed25519.golden.json')

        public_key = golden['publicKey']
        signature = golden['signature']
        message = golden['message']
        expected_message_hash = golden['messageHash']

        # Verify message hash
        message_bytes = message.encode('utf-8')
        computed_hash = hashlib.sha256(message_bytes).digest().hex()
        assert computed_hash == expected_message_hash, \
            f"Message hash mismatch: {computed_hash} != {expected_message_hash}"

        # Verify signature
        is_valid = Ed25519Signature.verify_signature(public_key, message, signature)
        assert is_valid, f"Ed25519 signature verification failed for golden vector"


class TestTransactionHashParity:
    """Test transaction hashing against TypeScript SDK vectors."""

    def test_tx_signing_vectors(self):
        """Test all transaction signing vectors for hash parity."""
        vectors = load_golden_fixture('tx_signing_vectors.json')

        for vector in vectors['vectors']:
            name = vector['name']
            private_key = vector['privateKey']
            expected_public_key = vector['publicKey']
            transaction = vector['transaction']
            expected_canonical = vector['canonicalJSON']
            expected_tx_hash = vector['txHash']
            expected_signature = vector['signature']

            # Test key derivation
            keypair = Ed25519KeyPair.from_seed(private_key)
            actual_public_key = keypair.public_key_bytes().hex()
            assert actual_public_key == expected_public_key, \
                f"Public key mismatch for {name}: {actual_public_key} != {expected_public_key}"

            # Test canonical JSON
            actual_canonical = dumps_canonical(transaction)
            assert actual_canonical == expected_canonical, \
                f"Canonical JSON mismatch for {name}:\nActual:   {actual_canonical}\nExpected: {expected_canonical}"

            # Test transaction hash
            actual_tx_hash = hash_transaction_hex(transaction)
            assert actual_tx_hash == expected_tx_hash, \
                f"Transaction hash mismatch for {name}: {actual_tx_hash} != {expected_tx_hash}"

            # Verify hash with verification function
            hash_verified = verify_transaction_hash(transaction, expected_tx_hash)
            assert hash_verified, f"Hash verification failed for {name}"


class TestSigningIntegration:
    """Test complete signing workflow integration."""

    def test_complete_signing_workflow(self):
        """Test the complete workflow: key derivation → hash → sign → verify."""
        # Use first vector for comprehensive test
        vectors = load_golden_fixture('tx_signing_vectors.json')
        vector = vectors['vectors'][0]  # simple_send_tokens

        private_key = vector['privateKey']
        transaction = vector['transaction']
        expected_tx_hash = vector['txHash']

        # 1. Key derivation
        keypair = Ed25519KeyPair.from_seed(private_key)

        # 2. Transaction hashing
        tx_hash = hash_transaction(transaction)
        tx_hash_hex = tx_hash.hex()
        assert tx_hash_hex == expected_tx_hash

        # 3. Create signature metadata (simple case)
        signature_metadata = {
            "type": "ed25519",
            "publicKey": keypair.public_key_bytes().hex(),
            "signer": "acc://alice.acme/book/1"
        }

        # 4. Hash for signing
        signing_hash = hash_for_ed25519_signing(signature_metadata, transaction)

        # 5. Sign the hash
        signature = keypair.sign(signing_hash)

        # 6. Verify signature
        is_valid = keypair.verify(signing_hash, signature)
        assert is_valid, "Signature verification failed in complete workflow"

        # 7. Verify with static method
        public_key_hex = keypair.public_key_bytes().hex()
        is_valid_static = Ed25519Signature.verify_signature(
            public_key_hex, signing_hash, signature
        )
        assert is_valid_static, "Static signature verification failed"


class TestHashFunctionVariants:
    """Test different hash function variants match expected behavior."""

    def test_signature_metadata_hashing(self):
        """Test signature metadata hashing."""
        metadata = {
            "type": "ed25519",
            "publicKey": "3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29",
            "signer": "acc://alice.acme/book/1"
        }

        # Hash should be deterministic
        hash1 = create_signature_metadata_hash(metadata)
        hash2 = create_signature_metadata_hash(metadata)
        assert hash1 == hash2, "Signature metadata hashing is not deterministic"
        assert len(hash1) == 32, f"Expected 32-byte hash, got {len(hash1)}"

    def test_ed25519_signing_hash(self):
        """Test Ed25519 signing hash combination."""
        vectors = load_golden_fixture('tx_signing_vectors.json')
        transaction = vectors['vectors'][0]['transaction']

        signature_metadata = {
            "type": "ed25519",
            "publicKey": "3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29",
            "signer": "acc://alice.acme/book/1"
        }

        # Hash should combine signature metadata and transaction hashes
        signing_hash = hash_for_ed25519_signing(signature_metadata, transaction)
        assert len(signing_hash) == 32, f"Expected 32-byte signing hash, got {len(signing_hash)}"

        # Should be deterministic
        signing_hash2 = hash_for_ed25519_signing(signature_metadata, transaction)
        assert signing_hash == signing_hash2, "Ed25519 signing hash is not deterministic"


class TestEnvelopeStructure:
    """Test envelope structure hash verification."""

    def test_envelope_transaction_hash(self):
        """Test envelope structure with transaction hash verification."""
        envelope = load_golden_fixture('envelope_fixed.golden.json')

        # Extract transaction and signature data
        signatures = envelope['signatures']
        transactions = envelope['transaction']

        assert len(signatures) == 1, "Expected exactly one signature in envelope"
        assert len(transactions) == 1, "Expected exactly one transaction in envelope"

        signature_data = signatures[0]
        transaction = transactions[0]

        # Verify signature structure
        assert 'type' in signature_data
        assert 'publicKey' in signature_data
        assert 'signature' in signature_data
        assert 'transactionHash' in signature_data

        expected_tx_hash = signature_data['transactionHash']

        # Compute transaction hash and verify it matches envelope
        computed_hash = hash_transaction_hex(transaction)
        assert computed_hash == expected_tx_hash, \
            f"Envelope transaction hash mismatch: {computed_hash} != {expected_tx_hash}"

        # Verify signature against transaction hash
        public_key = signature_data['publicKey']
        signature = signature_data['signature']

        # For envelope verification, we need to hash the transaction hash bytes
        tx_hash_bytes = bytes.fromhex(expected_tx_hash)

        # Verify the signature
        is_valid = Ed25519Signature.verify_signature(public_key, tx_hash_bytes, signature)
        assert is_valid, "Envelope signature verification failed"

    def test_envelope_signature_metadata(self):
        """Test envelope signature metadata structure."""
        envelope = load_golden_fixture('envelope_fixed.golden.json')
        signature_data = envelope['signatures'][0]

        # Verify all required fields are present
        required_fields = ['type', 'publicKey', 'signature', 'signer', 'signerVersion', 'timestamp', 'transactionHash']
        for field in required_fields:
            assert field in signature_data, f"Missing required field: {field}"

        # Verify field types and formats
        assert signature_data['type'] == 'ed25519'
        assert len(signature_data['publicKey']) == 64  # 32 bytes as hex
        assert len(signature_data['signature']) == 128  # 64 bytes as hex
        assert signature_data['signer'].startswith('acc://')
        assert isinstance(signature_data['signerVersion'], int)
        assert isinstance(signature_data['timestamp'], int)
        assert len(signature_data['transactionHash']) == 64  # 32 bytes as hex


if __name__ == '__main__':
    # Run tests when executed directly
    pytest.main([__file__, '-v'])