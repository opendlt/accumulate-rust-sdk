# ⚠️ WARNING: TypeScript Test Vectors Are Unreliable

## DO NOT TRUST TypeScript SDK Test Vectors

The TypeScript test vectors in `tests/golden/typescript_sdk_vectors.json` contain **multiple critical encoding bugs** and should not be used as canonical reference.

## Confirmed Issues

1. **UVarint Encoding Errors**: JavaScript precision limits cause wrong encoding for large numbers
2. **JSON Canonicalization Bugs**: Nested objects incorrectly emptied, causing data loss
3. **String Length Inconsistencies**: UTF-16 vs UTF-8 vs Unicode character counting mismatches

## Use These Instead

✅ **Canonical Source**: Go implementation in [gitlab.com/accumulatenetwork/accumulate](https://gitlab.com/accumulatenetwork/accumulate) at `pkg/types/encoding/`
✅ **Verification Tool**: `accumulate-debug.exe verify` command
✅ **Reference Algorithms**: Go's `encoding/binary` package

## Impact

- **UVarint `4294967296`**: TS produces `[128, 0]` but canonical is `[128, 128, 128, 128, 16]`
- **JSON Canonicalization**: TS produces `{"to":[{}]}` but canonical preserves data
- **Protocol Compliance**: Our Rust implementation correctly follows Accumulate protocol

## Testing Strategy

- ✅ Test against Go canonical implementation
- ✅ Verify with accumulate-debug.exe tool
- ❌ Ignore TypeScript test vector failures (they're wrong)
- ✅ Focus on actual network protocol compatibility

See `ENCODING_REMEDIATION.md` for complete technical details.