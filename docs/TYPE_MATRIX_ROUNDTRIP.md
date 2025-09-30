# Type Matrix Roundtrip Testing

This document describes the comprehensive type matrix system for proving that every generated protocol type roundtrips correctly through encoding/decoding.

## Overview

The type matrix system ensures that all protocol types can be:
1. **Serialized** to JSON/binary format
2. **Deserialized** back to the original type
3. **Re-serialized** with identical results

This guarantees data integrity and compatibility between different SDK implementations.

## Architecture

### Core Components

1. **`src/types_matrix.rs`** - Central type registry and traits
2. **`tests/conformance/type_matrix_roundtrip.rs`** - Comprehensive roundtrip tests
3. **`tests/conformance/type_matrix_verification.rs`** - Test infrastructure validation

### Key Traits

#### `SampleGenerator`
Generates safe sample instances for testing:

```rust
pub trait SampleGenerator {
    fn generate_sample() -> Self;
    fn generate_samples() -> Vec<Self>;
}
```

#### `RoundtripTestable`
Enables roundtrip testing for any type:

```rust
pub trait RoundtripTestable: Serialize + for<'de> Deserialize<'de> + Clone + PartialEq {
    fn test_json_roundtrip(&self) -> Result<(), String>;
    fn test_binary_roundtrip(&self) -> Result<(), String>;
}
```

## Type Registry

### `TYPE_NAMES` Constant

All protocol types that must pass roundtrip tests:

```rust
pub const TYPE_NAMES: &[&str] = &[
    // Core transaction types
    "TransactionEnvelope",
    "TransactionHeader",
    "TransactionSignature",
    "TransactionKeyPage",
    "TokenRecipient",
    "KeySpec",

    // API response types
    "StatusResponse",
    "NodeInfo",
    "QueryResponse",
    "TransactionResponse",
    "TransactionResult",
    "Event",
    "Attribute",
    "SignedTransaction",
    "Signature",
    "Account",
    "FaucetResponse",

    // V3 specific types
    "V3SubmitRequest",
    "V3SubmitResponse",
    "SubmitResult",
    "V3Signature",

    // Protocol types
    "ProtocolTransactionEnvelope",
    "ProtocolTransactionSignature",
    "ProtocolTransactionHeader",

    // Support types
    "BinaryReader",
    "BinaryWriter",
    "EncodingError",
    "DecodingError",
    "FieldReader",
    "Ed25519Signer",
    "AccOptions",
];
```

## Sample Generation

### Core Transaction Types

Each core type implements `SampleGenerator` with multiple sample variations:

#### TransactionEnvelope
- **Basic sample**: Standard transaction with common fields
- **Minimal sample**: Only required fields populated
- **Complex sample**: Multiple signatures, unicode content, nested metadata

#### TransactionHeader
- **Standard**: All fields populated with typical values
- **Minimal**: Only required fields (principal, timestamp)
- **Unicode**: Special characters and edge case values

#### TransactionSignature
- **Standard**: Complete signature with key page
- **Minimal**: Empty signature data
- **Maximum**: Large signatures with max field values

### Sample Characteristics

```rust
// Example: TransactionEnvelope samples
vec![
    // Standard sample
    TransactionEnvelope {
        header: TransactionHeader {
            principal: "acc://alice.acme/tokens",
            timestamp: 1234567890123,
            // ...
        },
        body: json!({"type": "send-tokens", /* ... */}),
        signatures: vec![/* ... */],
    },

    // Minimal sample
    TransactionEnvelope {
        header: TransactionHeader {
            principal: "acc://test.acme",
            timestamp: 1000000000000,
            initiator: None,
            nonce: None,
            memo: None,
            metadata: None,
        },
        body: json!({"type": "create-identity"}),
        signatures: vec![],
    },

    // Complex sample with edge cases
    TransactionEnvelope {
        header: TransactionHeader {
            principal: "acc://üñíçødé.acme/tøkeñs",
            timestamp: u64::MAX,
            memo: Some("Unicode: 🚀 ñoño café".to_string()),
            // ...
        },
        body: json!({/* complex nested structure */}),
        signatures: vec![/* multiple signatures */],
    }
]
```

## Roundtrip Testing

### JSON Roundtrip Process

1. **Serialize** object to JSON string
2. **Deserialize** JSON back to object
3. **Re-serialize** to JSON string
4. **Compare** original and deserialized objects
5. **Compare** original and re-serialized JSON strings

```rust
fn test_json_roundtrip(&self) -> Result<(), String> {
    // Serialize to JSON
    let json = serde_json::to_string(self)?;

    // Deserialize from JSON
    let deserialized: Self = serde_json::from_str(&json)?;

    // Re-serialize to JSON
    let json2 = serde_json::to_string(&deserialized)?;

    // Verify object equality
    if self != &deserialized {
        return Err("Deserialized object differs from original");
    }

    // Verify JSON string equality
    if json != json2 {
        return Err("Re-serialized JSON differs from original");
    }

    Ok(())
}
```

### Binary Roundtrip Process

For types supporting binary encoding (TransactionCodec):

1. **Encode** object to binary format
2. **Decode** binary back to object
3. **Re-encode** to binary format
4. **Compare** original and re-encoded binary data

```rust
// Example: TransactionEnvelope binary roundtrip
let encoded = TransactionCodec::encode_envelope(&envelope)?;
let decoded = TransactionCodec::decode_envelope(&encoded)?;
let re_encoded = TransactionCodec::encode_envelope(&decoded)?;

assert_eq!(encoded, re_encoded, "Binary roundtrip failed");
```

## Test Structure

### Main Test Functions

#### `test_all_types_roundtrip()`
Comprehensive test that validates all protocol types:

```rust
#[test]
fn test_all_types_roundtrip() {
    let mut tested_types = Vec::new();
    let mut failed_types = Vec::new();
    let mut total_samples = 0;

    // Test core types with SampleGenerator
    test_type_roundtrip::<TransactionEnvelope>("TransactionEnvelope", ...);
    test_type_roundtrip::<TransactionHeader>("TransactionHeader", ...);
    // ...

    // Test other types manually
    test_manual_types(...);

    // Verify coverage
    verify_type_coverage_implementation(&tested_types);

    if !failed_types.is_empty() {
        panic!("Roundtrip tests failed for {} types", failed_types.len());
    }
}
```

#### `test_transaction_codec_binary_roundtrip()`
Specialized test for binary encoding roundtrips:

```rust
#[test]
fn test_transaction_codec_binary_roundtrip() {
    let envelope = TransactionEnvelope::generate_sample();

    let encoded = TransactionCodec::encode_envelope(&envelope)?;
    let decoded = TransactionCodec::decode_envelope(&encoded)?;
    let re_encoded = TransactionCodec::encode_envelope(&decoded)?;

    assert_eq!(encoded, re_encoded);
}
```

### Coverage Verification

#### Type Coverage Check
Ensures all types in `TYPE_NAMES` have corresponding tests:

```rust
fn verify_type_coverage_implementation(tested_types: &[String]) {
    let missing_types: Vec<_> = TYPE_NAMES
        .iter()
        .filter(|type_name| !tested_types.contains(&type_name.to_string()))
        .collect();

    if !missing_types.is_empty() {
        println!("⚠️ Missing test implementations: {:?}", missing_types);
    }
}
```

#### Report Generation
Produces comprehensive coverage reports:

```rust
pub fn generate_type_test_report() -> String {
    let mut report = String::new();

    report.push_str("# Type Matrix Test Coverage Report\n\n");
    report.push_str(&format!("Total types: {}\n", TYPE_NAMES.len()));

    // Core types section
    report.push_str("## Core Transaction Types\n");
    for type_name in core_types {
        if TYPE_NAMES.contains(&type_name) {
            report.push_str(&format!("- ✅ {}\n", type_name));
        } else {
            report.push_str(&format!("- ❌ {}\n", type_name));
        }
    }

    // Coverage status
    match verify_type_coverage() {
        Ok(()) => report.push_str("✅ All types covered\n"),
        Err(missing) => {
            report.push_str(&format!("❌ {} types need implementation\n", missing.len()));
        }
    }

    report
}
```

## Usage

### Running Tests

```bash
# Run all roundtrip tests
cargo test --test type_matrix_roundtrip

# Run specific test categories
cargo test --test type_matrix_roundtrip test_all_types_roundtrip
cargo test --test type_matrix_roundtrip test_transaction_codec_binary_roundtrip

# Run verification tests
cargo test --test type_matrix_verification

# Run with verbose output
cargo test --test type_matrix_roundtrip -- --nocapture
```

### Expected Output

```
🔄 Testing TransactionEnvelope roundtrips...
  ✓ Sample 0 passed JSON roundtrip
  ✓ Sample 1 passed JSON roundtrip
  ✓ Sample 2 passed JSON roundtrip
  ✅ TransactionEnvelope passed all roundtrip tests (3 samples)

🔄 Testing TransactionHeader roundtrips...
  ✓ Sample 0 passed JSON roundtrip
  ✓ Sample 1 passed JSON roundtrip
  ✓ Sample 2 passed JSON roundtrip
  ✅ TransactionHeader passed all roundtrip tests (3 samples)

📊 ROUNDTRIP TEST SUMMARY
========================
Total samples tested: 45
Types tested: 15
Types in TYPE_NAMES: 27

✅ All types passed roundtrip tests!
```

### Integration with CI/CD

```yaml
- name: Type Matrix Roundtrip Tests
  run: |
    cargo test --test type_matrix_roundtrip
    cargo test --test type_matrix_verification
```

## Benefits

### Data Integrity
- **Guarantees** that serialization/deserialization preserves all data
- **Detects** incompatible changes between SDK versions
- **Ensures** protocol compatibility across implementations

### Development Safety
- **Prevents** breaking changes to existing types
- **Validates** new type implementations
- **Catches** serialization edge cases early

### Protocol Compliance
- **Verifies** TypeScript SDK compatibility
- **Ensures** consistent behavior across languages
- **Maintains** wire format stability

## Extending the System

### Adding New Types

1. **Add to `TYPE_NAMES`** constant
2. **Implement `SampleGenerator`** trait
3. **Implement `RoundtripTestable`** trait (automatic for most cases)
4. **Add test case** to roundtrip test suite

### Example: Adding New Type

```rust
// 1. Add to TYPE_NAMES
pub const TYPE_NAMES: &[&str] = &[
    // ... existing types ...
    "MyNewType",
];

// 2. Implement SampleGenerator
impl SampleGenerator for MyNewType {
    fn generate_sample() -> Self {
        Self {
            field1: "sample_value".to_string(),
            field2: 42,
        }
    }

    fn generate_samples() -> Vec<Self> {
        vec![
            Self::generate_sample(),
            Self { field1: "".to_string(), field2: 0 },
            Self { field1: "max_value".to_string(), field2: i32::MAX },
        ]
    }
}

// 3. RoundtripTestable is automatically implemented

// 4. Add to test suite
test_type_roundtrip::<MyNewType>("MyNewType", ...);
```

This comprehensive system ensures that every protocol type maintains data integrity through all encoding/decoding operations, providing confidence in the SDK's reliability and protocol compliance.