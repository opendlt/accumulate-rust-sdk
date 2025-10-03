"""
Pure Ed25519 signatures for Accumulate protocol.

This module provides Ed25519 key generation, signing, and verification
that matches the TypeScript SDK implementation exactly.
Uses PyNaCl (NaCl/TweetNaCl) to match the JavaScript implementation.
"""

import hashlib
from typing import Optional, Tuple, Union

try:
    import nacl.signing
    import nacl.encoding
    import nacl.exceptions
    NACL_AVAILABLE = True
except ImportError:
    NACL_AVAILABLE = False

# Fallback to cryptography if PyNaCl is not available
from cryptography.hazmat.primitives import serialization
from cryptography.hazmat.primitives.asymmetric.ed25519 import Ed25519PrivateKey, Ed25519PublicKey
from cryptography.exceptions import InvalidSignature


class Ed25519KeyPair:
    """
    Ed25519 key pair with Accumulate-specific derivations.

    Matches the TypeScript SDK's ED25519Key class exactly,
    including key derivation from both 32-byte seeds and 64-byte keypairs.
    """

    def __init__(self, signing_key, verify_key=None):
        """
        Initialize key pair from NaCl signing key.

        Args:
            signing_key: NaCl SigningKey or cryptography Ed25519PrivateKey
            verify_key: NaCl VerifyKey (optional, derived from signing_key)
        """
        if NACL_AVAILABLE:
            if isinstance(signing_key, nacl.signing.SigningKey):
                self._signing_key = signing_key
                self._verify_key = verify_key or signing_key.verify_key
                # Also store cryptography key for compatibility
                private_bytes = bytes(signing_key)
                self._private_key = Ed25519PrivateKey.from_private_bytes(private_bytes)
                self._public_key = self._private_key.public_key()
            else:
                # Convert from cryptography if needed
                raise TypeError("Expected nacl.signing.SigningKey when PyNaCl is available")
        else:
            # Fallback to cryptography
            self._private_key = signing_key
            self._public_key = signing_key.public_key()

    @classmethod
    def generate(cls) -> 'Ed25519KeyPair':
        """
        Generate a new Ed25519 key pair.

        Returns:
            New Ed25519KeyPair instance

        Example:
            >>> keypair = Ed25519KeyPair.generate()
            >>> len(keypair.public_key_bytes())
            32
        """
        private_key = Ed25519PrivateKey.generate()
        return cls(private_key)

    @classmethod
    def from_seed_or_key(cls, seed_or_key: Union[bytes, str]) -> 'Ed25519KeyPair':
        """
        Create key pair from seed or key (matches TypeScript ED25519Key.from).

        This exactly matches the TypeScript implementation's make() method:
        - 64 bytes: { publicKey: seedOrKey.slice(32), secretKey: seedOrKey }
        - 32 bytes: nacl.sign.keyPair.fromSeed(seedOrKey)

        Args:
            seed_or_key: 32-byte seed or 64-byte keypair as bytes or hex string

        Returns:
            Ed25519KeyPair derived from input

        Raises:
            ValueError: If input is not 32 or 64 bytes

        Example:
            >>> # 32-byte seed
            >>> seed = bytes.fromhex("0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef")
            >>> keypair = Ed25519KeyPair.from_seed_or_key(seed)
            >>> len(keypair.public_key_bytes())
            32
        """
        if isinstance(seed_or_key, str):
            seed_or_key = bytes.fromhex(seed_or_key)

        if NACL_AVAILABLE:
            if len(seed_or_key) == 64:
                # 64-byte keypair: { publicKey: seedOrKey.slice(32), secretKey: seedOrKey }
                public_key = seed_or_key[32:]
                secret_key = seed_or_key
                signing_key = nacl.signing.SigningKey(seed_or_key[:32])
                verify_key = nacl.signing.VerifyKey(public_key)
                return cls(signing_key, verify_key)
            elif len(seed_or_key) == 32:
                # 32-byte seed: nacl.sign.keyPair.fromSeed(seedOrKey)
                signing_key = nacl.signing.SigningKey(seed_or_key)
                return cls(signing_key)
            else:
                raise ValueError(f"Expected 64 or 32 bytes, got {len(seed_or_key)}")
        else:
            # Fallback to cryptography
            if len(seed_or_key) == 64:
                # Use the first 32 bytes as seed for cryptography
                seed = seed_or_key[:32]
            elif len(seed_or_key) == 32:
                seed = seed_or_key
            else:
                raise ValueError(f"Expected 64 or 32 bytes, got {len(seed_or_key)}")

            private_key = Ed25519PrivateKey.from_private_bytes(seed)
            return cls(private_key)

    @classmethod
    def from_seed(cls, seed: Union[bytes, str]) -> 'Ed25519KeyPair':
        """Alias for from_seed_or_key for backward compatibility."""
        return cls.from_seed_or_key(seed)

    def public_key_bytes(self) -> bytes:
        """
        Get public key as 32 bytes.

        Returns:
            32-byte public key (matches TypeScript implementation)
        """
        if NACL_AVAILABLE:
            return bytes(self._verify_key.encode())
        else:
            return self._public_key.public_bytes(
                encoding=serialization.Encoding.Raw,
                format=serialization.PublicFormat.Raw
            )

    def private_key_bytes(self) -> bytes:
        """
        Get private key as 32 bytes.

        Returns:
            32-byte private key seed
        """
        return self._private_key.private_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PrivateFormat.Raw,
            encryption_algorithm=serialization.NoEncryption()
        )

    def sign(self, message: Union[bytes, str]) -> bytes:
        """
        Sign message with Ed25519 (matches Dart sign method).

        Returns raw 64-byte signature exactly as the Dart implementation
        does with signature.bytes from Ed25519().sign().

        Args:
            message: Message to sign as bytes or string

        Returns:
            64-byte Ed25519 signature

        Example:
            >>> keypair = Ed25519KeyPair.generate()
            >>> message = b"test message"
            >>> signature = keypair.sign(message)
            >>> len(signature)
            64
        """
        if isinstance(message, str):
            message = message.encode('utf-8')

        # Pure Ed25519 signature (not Ed25519ph)
        signature = self._private_key.sign(message)
        return signature

    def verify(self, message: Union[bytes, str], signature: Union[bytes, str]) -> bool:
        """
        Verify Ed25519 signature (matches Dart verify method).

        Args:
            message: Original message that was signed
            signature: 64-byte signature to verify

        Returns:
            True if signature is valid, False otherwise

        Example:
            >>> keypair = Ed25519KeyPair.generate()
            >>> message = b"test message"
            >>> signature = keypair.sign(message)
            >>> keypair.verify(message, signature)
            True
        """
        if isinstance(message, str):
            message = message.encode('utf-8')

        if isinstance(signature, str):
            signature = bytes.fromhex(signature)

        try:
            self._public_key.verify(signature, message)
            return True
        except InvalidSignature:
            return False

    def derive_lite_identity_url(self) -> str:
        """
        Derive Lite Identity URL using Accumulate algorithm.

        This matches the Dart implementation exactly:
        1. keyHash = SHA256(publicKey) for Ed25519
        2. Use first 20 bytes of keyHash
        3. Calculate checksum = SHA256(hex(keyHash[0:20]))[28:]
        4. Format: acc://<keyHash[0:20]><checksum>

        Returns:
            Lite Identity URL string

        Example:
            >>> keypair = Ed25519KeyPair.from_seed("0" * 64)  # hex string
            >>> url = keypair.derive_lite_identity_url()
            >>> url.startswith("acc://")
            True
        """
        public_key = self.public_key_bytes()

        # For Ed25519: keyHash = SHA256(publicKey)
        key_hash_full = hashlib.sha256(public_key).digest()

        # Use first 20 bytes
        key_hash_20 = key_hash_full[:20]

        # Convert to hex string
        key_str = key_hash_20.hex()

        # Calculate checksum = SHA256(hex(keyHash[0:20]))[28:]
        checksum_full = hashlib.sha256(key_str.encode('utf-8')).digest()
        checksum = checksum_full[28:].hex()

        # Format: acc://<keyHash[0:20]><checksum>
        return f"acc://{key_str}{checksum}"

    def derive_lite_token_account_url(self) -> str:
        """
        Derive Lite Token Account URL for ACME.

        This matches the Dart implementation: LTA = LID + "/ACME"

        Returns:
            Lite Token Account URL string

        Example:
            >>> keypair = Ed25519KeyPair.generate()
            >>> url = keypair.derive_lite_token_account_url()
            >>> url.endswith("/ACME")
            True
        """
        lid = self.derive_lite_identity_url()
        return f"{lid}/ACME"


class Ed25519Signature:
    """
    Ed25519 signature verification utilities.

    Provides static methods for signature verification without key pairs.
    """

    @staticmethod
    def verify_signature(
        public_key: Union[bytes, str],
        message: Union[bytes, str],
        signature: Union[bytes, str]
    ) -> bool:
        """
        Verify Ed25519 signature with public key.

        Args:
            public_key: 32-byte public key as bytes or hex string
            message: Message that was signed
            signature: 64-byte signature as bytes or hex string

        Returns:
            True if signature is valid, False otherwise

        Example:
            >>> # From golden fixture
            >>> pub_key = "dff03fddf03d29a1f45daf8e9f2bd7c68ee3f2989b0c6c3385946d20f04b4926"
            >>> message = "test message for signature verification"
            >>> sig = "cff669b816312fbac709f12b0d18a96bcab6a570c27b2d13f662a04afdfeb36f..."
            >>> Ed25519Signature.verify_signature(pub_key, message, sig)
            True
        """
        if isinstance(public_key, str):
            public_key = bytes.fromhex(public_key)

        if isinstance(message, str):
            message = message.encode('utf-8')

        if isinstance(signature, str):
            signature = bytes.fromhex(signature)

        try:
            ed25519_public_key = Ed25519PublicKey.from_public_bytes(public_key)
            ed25519_public_key.verify(signature, message)
            return True
        except (InvalidSignature, ValueError):
            return False

    @staticmethod
    def public_key_from_private(private_key: Union[bytes, str]) -> bytes:
        """
        Derive public key from private key.

        Args:
            private_key: 32-byte private key as bytes or hex string

        Returns:
            32-byte public key
        """
        if isinstance(private_key, str):
            private_key = bytes.fromhex(private_key)

        ed25519_private_key = Ed25519PrivateKey.from_private_bytes(private_key)
        public_key = ed25519_private_key.public_key()

        return public_key.public_bytes(
            encoding=serialization.Encoding.Raw,
            format=serialization.PublicFormat.Raw
        )


def verify_key_derivation(private_key_hex: str, expected_public_key_hex: str) -> bool:
    """
    Verify that private key derives to expected public key.

    This is useful for validating test vectors and ensuring compatibility
    with other SDK implementations.

    Args:
        private_key_hex: Private key as hex string
        expected_public_key_hex: Expected public key as hex string

    Returns:
        True if derivation matches, False otherwise

    Example:
        >>> # From tx_signing_vectors.json
        >>> private_key = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
        >>> expected_public = "3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29"
        >>> verify_key_derivation(private_key, expected_public)
        True
    """
    try:
        keypair = Ed25519KeyPair.from_seed(private_key_hex)
        actual_public = keypair.public_key_bytes().hex()
        return actual_public == expected_public_key_hex
    except Exception:
        return False


if __name__ == '__main__':
    # Test basic functionality
    print("Testing Ed25519 implementation...")

    # Test key generation
    keypair = Ed25519KeyPair.generate()
    print(f"Generated keypair - public key: {keypair.public_key_bytes().hex()}")

    # Test signing and verification
    message = b"test message for signature verification"
    signature = keypair.sign(message)
    is_valid = keypair.verify(message, signature)
    print(f"Sign/verify test: {is_valid}")

    # Test deterministic key derivation from seed
    seed = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    deterministic_keypair = Ed25519KeyPair.from_seed(seed)
    public_key_hex = deterministic_keypair.public_key_bytes().hex()
    print(f"Deterministic public key: {public_key_hex}")

    # Test URL derivation
    lid_url = deterministic_keypair.derive_lite_identity_url()
    lta_url = deterministic_keypair.derive_lite_token_account_url()
    print(f"LID URL: {lid_url}")
    print(f"LTA URL: {lta_url}")

    # Test against known vector from tx_signing_vectors.json
    expected_public = "3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29"
    matches = verify_key_derivation(seed, expected_public)
    print(f"Vector validation: {matches}")

    print("Ed25519 implementation test complete!")