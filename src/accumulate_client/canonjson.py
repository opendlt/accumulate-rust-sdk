"""
Canonical JSON encoding that matches Dart/TypeScript SDK exactly.

This module provides canonical JSON serialization with deterministic key ordering
and exact formatting to ensure bit-for-bit compatibility across implementations.

The implementation follows the same semantics as Dart's json_canonical.dart:
- Object keys are sorted lexicographically by raw UTF-16 code units
- No insignificant whitespace (separators=(',', ':'))
- Integers preserved as integers (no float coercion)
- String escaping follows JSON spec exactly
- Base64 encoding for bytes where applicable
"""

import json
import base64
from typing import Any, Dict, List, Union
from collections import OrderedDict


def dumps_canonical(obj: Any) -> str:
    """
    Serialize object to canonical JSON string.

    This function produces output identical to the Dart SDK's canonicalJsonString()
    and TypeScript SDK's canonical JSON implementation.

    Args:
        obj: Python object to serialize (dict, list, str, int, float, bool, None)

    Returns:
        Canonical JSON string with sorted keys and no extra whitespace

    Examples:
        >>> dumps_canonical({"z": 3, "a": 1, "m": 2})
        '{"a":1,"m":2,"z":3}'

        >>> dumps_canonical({"b": 2, "a": {"d": 4, "c": 3}})
        '{"a":{"c":3,"d":4},"b":2}'
    """
    canonicalized = _canonicalize(obj)
    return json.dumps(canonicalized, separators=(',', ':'), ensure_ascii=False, sort_keys=False)


def _canonicalize(value: Any) -> Any:
    """
    Recursively canonicalize a value by sorting object keys and handling special types.

    This mirrors the Dart implementation's _canonicalize function exactly:
    - Maps (dicts) have their keys sorted lexicographically
    - Lists (arrays) are recursively canonicalized
    - Primitive types (numbers, strings, bool, null) pass through unchanged
    - Bytes objects are encoded as base64 strings if needed

    Args:
        value: Value to canonicalize

    Returns:
        Canonicalized value ready for JSON serialization
    """
    if isinstance(value, dict):
        # Sort keys lexicographically like Dart does
        # Convert all keys to strings and sort them
        sorted_keys = sorted(value.keys(), key=lambda k: str(k))

        # Build new ordered dict with canonicalized values
        canonicalized_dict = OrderedDict()
        for key in sorted_keys:
            canonicalized_dict[str(key)] = _canonicalize(value[key])

        return canonicalized_dict

    elif isinstance(value, (list, tuple)):
        # Recursively canonicalize list elements
        return [_canonicalize(item) for item in value]

    elif isinstance(value, bytes):
        # Encode bytes as base64 string to match how Dart/TS represent binary data
        return base64.b64encode(value).decode('ascii')

    elif isinstance(value, (str, int, float, bool, type(None))):
        # Primitive types pass through unchanged
        # This preserves int vs float distinction which is important
        return value

    else:
        # For any other type, convert to string representation
        # This handles edge cases while maintaining compatibility
        return str(value)


def verify_canonical_format(canonical_str: str) -> bool:
    """
    Verify that a string is in canonical JSON format.

    This checks that:
    - The string parses as valid JSON
    - Re-canonicalizing produces the same string
    - No extra whitespace is present

    Args:
        canonical_str: JSON string to verify

    Returns:
        True if string is in canonical format, False otherwise
    """
    try:
        # Parse the JSON
        parsed = json.loads(canonical_str)

        # Re-canonicalize and compare
        recanonical = dumps_canonical(parsed)

        return canonical_str == recanonical
    except (json.JSONDecodeError, TypeError, ValueError):
        return False


def canonicalize_for_hashing(obj: Any) -> bytes:
    """
    Canonicalize object and return UTF-8 bytes suitable for hashing.

    This is a convenience function for the common pattern of canonicalizing
    an object and then encoding it as UTF-8 bytes for hash computation.

    Args:
        obj: Object to canonicalize

    Returns:
        UTF-8 encoded canonical JSON bytes
    """
    canonical_str = dumps_canonical(obj)
    return canonical_str.encode('utf-8')


# Additional utilities for testing and validation

def compare_with_dart_output(obj: Any, expected_dart_canonical: str) -> bool:
    """
    Compare Python canonical output with expected Dart SDK output.

    This is useful for testing compatibility with golden fixtures.

    Args:
        obj: Python object to canonicalize
        expected_dart_canonical: Expected canonical JSON from Dart SDK

    Returns:
        True if outputs match exactly, False otherwise
    """
    python_canonical = dumps_canonical(obj)
    return python_canonical == expected_dart_canonical


def debug_canonicalization(obj: Any) -> Dict[str, Any]:
    """
    Debug helper that shows canonicalization steps.

    Returns detailed information about the canonicalization process
    for debugging mismatches with other implementations.

    Args:
        obj: Object to analyze

    Returns:
        Dictionary with canonicalization details
    """
    canonicalized = _canonicalize(obj)
    canonical_str = dumps_canonical(obj)

    return {
        'original': obj,
        'canonicalized': canonicalized,
        'canonical_string': canonical_str,
        'utf8_bytes': canonical_str.encode('utf-8'),
        'byte_length': len(canonical_str.encode('utf-8')),
        'is_valid_json': verify_canonical_format(canonical_str)
    }


if __name__ == '__main__':
    # Basic test cases when run as script
    test_cases = [
        {"z": 3, "a": 1, "m": 2},
        {"b": 2, "a": {"d": 4, "c": 3}},
        {"arr": [{"y": 2, "x": 1}, {"b": 0}, {"a": 0}]},
        {"string": "test", "number": 42, "boolean": True, "null": None}
    ]

    print("Canonical JSON Test Cases:")
    print("=" * 50)

    for i, test_case in enumerate(test_cases):
        canonical = dumps_canonical(test_case)
        print(f"Test {i+1}:")
        print(f"Input:     {test_case}")
        print(f"Canonical: {canonical}")
        print(f"Valid:     {verify_canonical_format(canonical)}")
        print()