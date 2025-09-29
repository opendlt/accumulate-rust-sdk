"""
Test canonical JSON parity with Dart/TypeScript SDK implementations.

This test suite validates that the Python canonical JSON implementation produces
identical output to the Dart and TypeScript SDKs using golden fixture files.
"""

import json
import hashlib
import os
import sys
from pathlib import Path
import pytest

# Add src to path so we can import the accumulate_client module
sys.path.insert(0, str(Path(__file__).parent.parent.parent / "src"))

from accumulate_client.canonjson import dumps_canonical, canonicalize_for_hashing, compare_with_dart_output


class TestCanonicalJsonParity:
    """Test canonical JSON implementation against golden fixtures."""

    @classmethod
    def setup_class(cls):
        """Load golden fixture files."""
        cls.golden_dir = Path(__file__).parent.parent / "golden"

        # Load canonical JSON test cases
        canonical_json_file = cls.golden_dir / "canonical_json_tests.json"
        if canonical_json_file.exists():
            with open(canonical_json_file, 'r', encoding='utf-8') as f:
                cls.canonical_json_fixtures = json.load(f)
        else:
            cls.canonical_json_fixtures = {"testCases": []}

        # Load transaction signing vectors
        tx_signing_file = cls.golden_dir / "tx_signing_vectors.json"
        if tx_signing_file.exists():
            with open(tx_signing_file, 'r', encoding='utf-8') as f:
                cls.tx_signing_fixtures = json.load(f)
        else:
            cls.tx_signing_fixtures = {"vectors": []}

        # Load envelope fixtures
        envelope_file = cls.golden_dir / "envelope_fixed.golden.json"
        if envelope_file.exists():
            with open(envelope_file, 'r', encoding='utf-8') as f:
                cls.envelope_fixtures = json.load(f)
        else:
            cls.envelope_fixtures = {}

        # Try to load ts_parity_fixtures.json if it exists
        ts_parity_file = cls.golden_dir / "ts_parity_fixtures.json"
        if ts_parity_file.exists():
            with open(ts_parity_file, 'r', encoding='utf-8') as f:
                cls.ts_parity_fixtures = json.load(f)
        else:
            cls.ts_parity_fixtures = {"fixtures": []}

    def test_basic_canonical_json_cases(self):
        """Test basic canonical JSON formatting against known expected outputs."""
        test_cases = [
            # Simple object with unsorted keys
            (
                {"z": 3, "a": 1, "m": 2},
                '{"a":1,"m":2,"z":3}'
            ),
            # Nested object
            (
                {"z": {"y": 2, "x": 1}, "a": 1},
                '{"a":1,"z":{"x":1,"y":2}}'
            ),
            # Array with objects
            (
                {"arr": [{"b": 2, "a": 1}, {"d": 4, "c": 3}]},
                '{"arr":[{"a":1,"b":2},{"c":3,"d":4}]}'
            ),
            # All primitive types
            (
                {"string": "test", "number": 42, "boolean": True, "null": None},
                '{"boolean":true,"null":null,"number":42,"string":"test"}'
            )
        ]

        for test_input, expected in test_cases:
            result = dumps_canonical(test_input)
            assert result == expected, f"Mismatch for {test_input}: got {result}, expected {expected}"

    def test_canonical_json_fixtures(self):
        """Test against canonical JSON golden fixtures."""
        if not self.canonical_json_fixtures.get("testCases"):
            pytest.skip("No canonical JSON fixtures found")

        for test_case in self.canonical_json_fixtures["testCases"]:
            name = test_case["name"]
            input_obj = test_case["input"]
            expected = test_case["expectedCanonical"]

            result = dumps_canonical(input_obj)
            assert result == expected, f"Test case {name} failed: got {result}, expected {expected}"

    def test_transaction_signing_vectors(self):
        """Test canonical JSON against transaction signing vectors."""
        if not self.tx_signing_fixtures.get("vectors"):
            pytest.skip("No transaction signing fixtures found")

        for vector in self.tx_signing_fixtures["vectors"]:
            name = vector["name"]
            transaction = vector["transaction"]
            expected_canonical = vector["canonicalJSON"]
            expected_hash = vector["txHash"]

            # Test canonical JSON
            result_canonical = dumps_canonical(transaction)
            assert result_canonical == expected_canonical, \
                f"Canonical JSON mismatch for {name}: got {result_canonical}, expected {expected_canonical}"

            # Test hash computation
            canonical_bytes = canonicalize_for_hashing(transaction)
            computed_hash = hashlib.sha256(canonical_bytes).hexdigest()
            assert computed_hash == expected_hash, \
                f"Hash mismatch for {name}: got {computed_hash}, expected {expected_hash}"

    def test_envelope_fixtures(self):
        """Test canonical JSON with envelope fixtures."""
        if not self.envelope_fixtures:
            pytest.skip("No envelope fixtures found")

        # Test transaction part of envelope
        if "transaction" in self.envelope_fixtures and self.envelope_fixtures["transaction"]:
            tx = self.envelope_fixtures["transaction"][0]  # First transaction
            canonical = dumps_canonical(tx)

            # Verify it's valid JSON and can round-trip
            reparsed = json.loads(canonical)
            recanonical = dumps_canonical(reparsed)
            assert canonical == recanonical, "Canonical JSON should be stable under round-trip"

    def test_ts_parity_fixtures(self):
        """Test against TypeScript SDK parity fixtures if available."""
        if not self.ts_parity_fixtures.get("fixtures"):
            pytest.skip("No TypeScript parity fixtures found")

        for fixture in self.ts_parity_fixtures["fixtures"]:
            if "canonical_json_string" in fixture:
                py_obj = fixture.get("object") or fixture.get("input")
                expected_canonical = fixture["canonical_json_string"]

                if py_obj is not None:
                    result = dumps_canonical(py_obj)
                    assert result == expected_canonical, \
                        f"TS parity mismatch: got {result}, expected {expected_canonical}"

                    # Test hash if provided
                    if "hash" in fixture:
                        canonical_bytes = canonicalize_for_hashing(py_obj)
                        computed_hash = hashlib.sha256(canonical_bytes).hexdigest()
                        expected_hash = fixture["hash"]
                        assert computed_hash == expected_hash, \
                            f"Hash mismatch: got {computed_hash}, expected {expected_hash}"

    def test_key_sorting_unicode(self):
        """Test that key sorting handles Unicode correctly."""
        # Test Unicode key sorting
        test_obj = {
            "β": 2,  # Greek beta
            "a": 1,  # ASCII
            "α": 0,  # Greek alpha
            "z": 3   # ASCII
        }

        result = dumps_canonical(test_obj)
        # Keys should be sorted by their string representation
        # In lexicographic order: "a", "z", "α", "β" (ASCII before Greek)
        expected = '{"a":1,"z":3,"α":0,"β":2}'
        assert result == expected

    def test_number_preservation(self):
        """Test that integers are preserved as integers, not converted to floats."""
        test_obj = {
            "int": 42,
            "float": 3.14,
            "zero": 0,
            "negative": -123
        }

        result = dumps_canonical(test_obj)
        expected = '{"float":3.14,"int":42,"negative":-123,"zero":0}'
        assert result == expected

        # Verify that integers don't have decimal points
        assert ".0" not in result or "3.14" in result  # Only float should have decimal

    def test_string_escaping(self):
        """Test that string escaping follows JSON spec exactly."""
        test_cases = [
            {"quote": '"hello"'},
            {"backslash": "back\\slash"},
            {"newline": "line\nbreak"},
            {"tab": "tab\there"},
            {"unicode": "emoji🌍test"}
        ]

        for test_obj in test_cases:
            result = dumps_canonical(test_obj)
            # Should be valid JSON
            reparsed = json.loads(result)
            assert reparsed == test_obj

    def test_bytes_handling(self):
        """Test that bytes are handled appropriately (if present in test data)."""
        # Only test if bytes are present in the actual data
        test_obj = {"data": b"hello world"}
        result = dumps_canonical(test_obj)

        # Should be valid JSON with base64 encoded bytes
        reparsed = json.loads(result)
        assert "data" in reparsed

    def test_empty_and_null_values(self):
        """Test handling of empty and null values."""
        test_cases = [
            {},  # Empty object
            [],  # Empty array
            {"empty": {}},  # Object with empty object
            {"null": None},  # Null value
            {"empty_array": []},  # Empty array value
            {"empty_string": ""}  # Empty string
        ]

        for test_obj in test_cases:
            result = dumps_canonical(test_obj)
            # Should be valid JSON and round-trip correctly
            reparsed = json.loads(result)
            recanonical = dumps_canonical(reparsed)
            assert result == recanonical

    def test_deep_nesting(self):
        """Test canonical JSON with deeply nested structures."""
        deep_obj = {
            "level1": {
                "level2": {
                    "level3": {
                        "z": 3,
                        "a": 1,
                        "m": 2
                    }
                }
            }
        }

        result = dumps_canonical(deep_obj)
        expected = '{"level1":{"level2":{"level3":{"a":1,"m":2,"z":3}}}}'
        assert result == expected

    def test_array_with_mixed_types(self):
        """Test arrays containing mixed data types."""
        test_obj = {
            "mixed_array": [
                {"z": 2, "a": 1},  # Object
                "string",           # String
                42,                 # Integer
                3.14,              # Float
                True,              # Boolean
                None,              # Null
                [1, 2, 3]         # Nested array
            ]
        }

        result = dumps_canonical(test_obj)
        # Should be valid JSON
        reparsed = json.loads(result)
        assert reparsed["mixed_array"][0] == {"a": 1, "z": 2}  # Object keys sorted

    def test_roundtrip_stability(self):
        """Test that canonical JSON is stable under parse/serialize cycles."""
        complex_obj = {
            "z": [
                {"c": 3, "a": 1, "b": 2},
                {"f": 6, "d": 4, "e": 5}
            ],
            "a": {
                "nested": {
                    "y": 2,
                    "x": 1
                }
            },
            "m": "middle"
        }

        # Multiple rounds of canonicalization should be stable
        canonical1 = dumps_canonical(complex_obj)
        parsed = json.loads(canonical1)
        canonical2 = dumps_canonical(parsed)
        parsed2 = json.loads(canonical2)
        canonical3 = dumps_canonical(parsed2)

        assert canonical1 == canonical2 == canonical3

    def test_compare_with_dart_output_utility(self):
        """Test the utility function for comparing with Dart output."""
        test_obj = {"z": 3, "a": 1, "m": 2}
        dart_output = '{"a":1,"m":2,"z":3}'

        assert compare_with_dart_output(test_obj, dart_output)

        # Should return False for incorrect output
        wrong_output = '{"a":1,"m":2,"z":4}'
        assert not compare_with_dart_output(test_obj, wrong_output)


def test_standalone_example():
    """Test that can be run independently to verify basic functionality."""
    # This test doesn't rely on fixtures and can verify basic functionality
    test_input = {
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

    expected_canonical = '{"body":{"to":[{"amount":"1000","url":"acc://bob.acme/tokens"}],"type":"send-tokens"},"header":{"principal":"acc://alice.acme/tokens","timestamp":1234567890123}}'
    expected_hash = "4be49c59c717f1984646998cecac0e5225378d9bbe2e18928272a85b7dfcb608"

    result_canonical = dumps_canonical(test_input)
    assert result_canonical == expected_canonical

    # Test hash
    canonical_bytes = canonicalize_for_hashing(test_input)
    computed_hash = hashlib.sha256(canonical_bytes).hexdigest()
    assert computed_hash == expected_hash


if __name__ == "__main__":
    # Run basic tests when executed directly
    pytest.main([__file__, "-v"])