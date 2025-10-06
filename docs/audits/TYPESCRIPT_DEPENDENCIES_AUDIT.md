# Audit: TypeScript Dependencies in Test Suites

## 🚨 Current State: Tests Still Depend on Buggy TypeScript Vectors

### Files Still Using TypeScript Test Vectors

#### 1. `tests/binary_parity.rs` ❌ **CRITICAL**
- **Function**: `load_test_vectors()` loads `typescript_sdk_vectors.json`
- **Tests Affected**:
  - `test_uvarint_parity()` - ❌ Uses buggy TypeScript UVarint vectors
  - `test_varint_parity()` - ✅ Actually passing now (canonical encoding works)
  - `test_string_parity()` - ✅ Passing (fixed UTF-16 vs Unicode issues)
  - `test_bytes_parity()` - Status unknown
  - `test_boolean_parity()` - Status unknown
  - `test_hash_parity()` - Status unknown
  - `test_canonical_json_parity()` - ❌ Uses buggy empty object vectors
  - `test_envelope_hash_parity()` - ❌ Uses buggy JSON canonicalization
  - `benchmark_encoding_performance()` - ❌ Performance tests on wrong data

#### 2. `tests/conformance/binary_parity.rs` ❌ **NEEDS REVIEW**
- **Function**: `load_typescript_vectors()` loads `typescript_sdk_vectors.json`
- **Status**: Need to check what tests are using this

#### 3. `tests/golden/typescript_sdk_vectors.json` ❌ **SOURCE OF BUGS**
- **Size**: Large JSON file with test vectors
- **Problems**: Contains the encoding bugs we identified
- **Impact**: Any test loading this file is potentially unreliable

### Tests That Should Be Replaced/Updated

#### High Priority (Known Bugs)
1. **UVarint Tests**:
   - Problem: `4294967296` encoded as `[128, 0]` instead of correct `[128, 128, 128, 128, 16]`
   - Solution: Use Go canonical test vectors or generate from Go implementation

2. **JSON Canonicalization Tests**:
   - Problem: Nested objects emptied to `{}` losing transaction data
   - Solution: Use standard canonical JSON test cases

3. **Envelope Hash Tests**:
   - Problem: Wrong signing payload format
   - Solution: Generate test vectors from Accumulate debug tool

#### Lower Priority (May Be Correct)
- Bytes parity tests
- Boolean parity tests
- Hash parity tests
- VarInt tests (these are actually passing now)
- String tests (these are actually passing now)

### Recommended Remediation Strategy

#### Phase 1: Disable Problematic Tests
```rust
#[cfg(feature = "typescript-compat-tests")]
#[test]
fn test_uvarint_parity() {
    // Only run when explicitly testing TypeScript compatibility
}

#[cfg(feature = "typescript-compat-tests")]
#[test]
fn test_canonical_json_parity() {
    // Only run when explicitly testing TypeScript compatibility
}
```

#### Phase 2: Create Canonical Test Vectors
Generate new test vectors from:
1. **Accumulate debug tool** output for transactions/envelopes
2. **Go implementation** for primitive encoding (UVarint, VarInt, etc.)
3. **RFC standards** for JSON canonicalization

#### Phase 3: Replace Tests
```rust
// Replace TypeScript vectors with canonical vectors
fn load_canonical_test_vectors() -> CanonicalTestVectors {
    // Load from go-generated or rfc-compliant test vectors
}

#[test]
fn test_canonical_uvarint_encoding() {
    // Test against Go binary.PutUvarint reference
}

#[test]
fn test_canonical_json_canonicalization() {
    // Test against RFC 7517 / standard sorted-key JSON
}
```

### Files That Should Be Safe (Not Using TypeScript Vectors)

#### ✅ These tests should be reliable:
- `tests/conformance/hash_vectors_test.rs` - May use independent hash vectors
- `tests/conformance/signing_vectors_test.rs` - May use independent signing vectors
- `tests/conformance/url_derivation_test.rs` - URL derivation logic
- Tests in `/conformance/` subdirectory (need verification)

### Immediate Actions Needed

#### 1. Feature Flag Problematic Tests
Add to `Cargo.toml`:
```toml
[features]
default = ["async-client"]
typescript-compat-tests = []  # Optional TypeScript compatibility tests
```

#### 2. Update Test Comments
Change from:
```rust
//! These tests ensure bit-for-bit compatibility between the Rust SDK
//! and TypeScript SDK binary encoding/decoding implementations.
```

To:
```rust
//! WARNING: These tests use potentially buggy TypeScript test vectors.
//! See TYPESCRIPT_VECTORS_WARNING.md for details.
//! Use canonical tests in /conformance/ directory for reliable validation.
```

#### 3. Add Canonical Tests
Create `tests/canonical/` directory with Go-generated or RFC-compliant test vectors.

### Current Test Run Status

When running `cargo test`, these problematic tests are executed by default:
- ❌ `test_uvarint_parity` - FAILS (known TypeScript bug)
- ❌ `test_canonical_json_parity` - FAILS (known TypeScript bug)
- ❌ `test_envelope_hash_parity` - FAILS (known TypeScript bug)
- ✅ `test_varint_parity` - PASSES (canonical encoding works)
- ✅ `test_string_parity` - PASSES (canonical encoding works)

### Risk Assessment

**HIGH RISK**: Developers may think our implementation is broken when they see test failures, not realizing the test vectors themselves are buggy.

**MEDIUM RISK**: Performance benchmarks based on wrong data may give misleading results.

**LOW RISK**: Some tests may be passing by coincidence even with buggy vectors.

## Recommendation

**Immediate**: Feature-flag the known problematic tests so they don't run by default.
**Short-term**: Create canonical test suite based on Go implementation.
**Long-term**: Phase out TypeScript vector dependencies entirely.