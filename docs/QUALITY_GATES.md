# Quality Gates for Accumulate Rust SDK

This document describes the quality gates enforced in the Accumulate Rust SDK to ensure code quality, completeness, and test coverage.

## Overview

The quality gates enforce:

1. **No Stubs/TODOs**: Prohibits incomplete code patterns in source files
2. **Coverage Gates**: Enforces minimum test coverage thresholds
3. **Clippy Lints**: Static analysis to catch common issues

## No Stubs / No TODOs

### Prohibited Patterns

The following patterns are prohibited in source code (`src/` and `examples/` directories):

#### Code Patterns
- `TODO` - Incomplete work markers
- `FIXME` - Code that needs fixing
- `XXX` - Problematic code markers
- `TBD` - To be determined markers
- `HACK` - Temporary workarounds
- `unimplemented!()` - Unimplemented functions
- `todo!()` - Placeholder macros
- `panic!("TODO")` - Panic with stub messages
- `panic!("FIXME")` - Panic with fix markers
- `panic!("Not implemented")` - Unimplemented panics

#### Comment Patterns
- `// TODO` - Comments with TODO markers
- `// FIXME` - Comments with FIXME markers
- `/* TODO` - Block comments with markers

### Test Implementation

The `tests/repo/no_todos.rs` test scans all `.rs` files in:
- `src/` directory (all source code)
- `examples/` directory (example code)

**Excluded directories**: `tests/`, `tooling/`, `target/`, `.git/`

### Context-Aware Detection

The scanner differentiates between:

- **Code Context**: All patterns prohibited
- **Comment Context**: Some patterns allowed (regular TODOs), others prohibited (`unimplemented!()`, `todo!()`)
- **String Context**: Generally allowed in string literals

### Running the Test

```bash
# Run no-stubs test
cargo test --test no_todos

# Run specific test functions
cargo test --test no_todos test_no_todos_in_source_code
cargo test --test no_todos test_prohibited_patterns_detection
```

### Clippy Integration

The project's `Cargo.toml` includes Clippy lints that complement the test:

```toml
[lints.clippy]
todo = "deny"
unimplemented = "deny"
panic = "deny"
```

## Coverage Gates

### Coverage Thresholds

- **Overall Coverage**: ≥ 70%
- **Critical Modules**: ≥ 85%

### Critical Modules

Critical modules requiring higher coverage:
- `src/codec/**` - Transaction encoding/decoding
- `src/crypto/**` - Cryptographic operations
- `src/canonjson.rs` - Canonical JSON implementation

### Coverage Script

The `scripts/coverage_gate.ps1` PowerShell script enforces coverage thresholds:

```powershell
# Run coverage gate with default thresholds
./scripts/coverage_gate.ps1

# Custom thresholds
./scripts/coverage_gate.ps1 -OverallThreshold 75 -CriticalThreshold 90

# Skip coverage generation, use existing report
./scripts/coverage_gate.ps1 -SkipGeneration

# Verbose output
./scripts/coverage_gate.ps1 -Verbose
```

### Coverage Reports

The script generates:
- **LCOV Report**: `target/lcov.info`
- **HTML Report**: `target/llvm-cov/html/index.html`
- **Console Summary**: Real-time coverage statistics

### Coverage Analysis

When coverage gates fail, the script provides:

1. **Overall vs Critical Coverage**: Separate tracking for different code areas
2. **Per-File Breakdown**: Coverage percentage for each critical file
3. **Low Coverage Hotspots**: Files with < 50% coverage (showing worst 10)
4. **Actionable Guidance**: Specific steps to improve coverage

## Running Quality Gates

### Prerequisites

Install required tools:

```bash
# Install cargo-llvm-cov (if not already installed)
cargo install cargo-llvm-cov --locked
```

### Individual Gates

```bash
# 1. Check for prohibited patterns
cargo test --test no_todos

# 2. Run coverage analysis
cargo llvm-cov --all-features --workspace --lcov --output-path target/lcov.info

# 3. Enforce coverage gates
./scripts/coverage_gate.ps1
```

### Combined Quality Check

```bash
# Run all quality gates in sequence
cargo test --test no_todos && ./scripts/coverage_gate.ps1
```

### Cargo Aliases

The project includes convenient aliases in `.cargo/config.toml`:

```bash
# Coverage collection
cargo cov              # Generate LCOV report
cargo cov-report       # Generate HTML report
cargo cov-open         # Open HTML report in browser
cargo cov-clean        # Clean coverage data

# Quality gates
cargo quality          # Run no_todos test
```

## CI/CD Integration

### GitHub Actions / CI

Example workflow step:

```yaml
- name: Quality Gates
  run: |
    # Check for prohibited patterns
    cargo test --test no_todos

    # Install coverage tool
    cargo install cargo-llvm-cov --locked

    # Run coverage gate
    pwsh ./scripts/coverage_gate.ps1
```

### Pre-commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
echo "Running quality gates..."

# Check for TODOs/stubs
cargo test --test no_todos || exit 1

# Check basic coverage (can be more lenient for pre-commit)
cargo llvm-cov --all-features --workspace > /dev/null || exit 1

echo "Quality gates passed!"
```

## Troubleshooting

### Common Issues

1. **"No cargo-llvm-cov found"**
   ```bash
   cargo install cargo-llvm-cov --locked
   ```

2. **"Coverage generation failed"**
   - Ensure all tests pass: `cargo test`
   - Clean previous coverage: `cargo llvm-cov clean`
   - Check for compilation errors

3. **"TODO/FIXME found in source"**
   - Replace with proper implementation
   - Move TODOs to issue tracking
   - Use proper error handling instead of panics

4. **"Low coverage in critical modules"**
   - Focus on `src/codec/**` and `src/crypto/**` first
   - Add unit tests for public functions
   - Test error conditions and edge cases

### Coverage Improvement Tips

1. **Unit Tests**: Test individual functions and modules
2. **Integration Tests**: Test component interactions
3. **Error Cases**: Test failure scenarios and edge cases
4. **Property-Based Tests**: Use fuzzing for complex logic
5. **Documentation Tests**: Include examples in doc comments

## Quality Gate Results

### Success Output

```
✅ QUALITY GATE PASSED: No prohibited patterns found in source code
   Files scanned: 45
   Directories: src, examples

✅ All coverage gates passed!
   Overall: 78.5% >= 70%
   Critical modules: All above 85%
```

### Failure Output

```
❌ QUALITY GATE FAILED: Found 3 prohibited patterns in source code
📁 File: src/crypto/example.rs
   Line 42: // TODO: implement advanced encryption (pattern: 'TODO')

💥 Coverage gates failed!
   Overall coverage: 65.2% < 70%
   Critical coverage: 82.1% < 85%

⚠️  Low Coverage Files (< 50%):
   ❌  23.1% src/utils/helper.rs
   ❌  31.8% src/client/retry.rs
```

## Configuration

### Cargo.toml Lints

```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
todo = "deny"
unimplemented = "deny"
panic = "deny"
unwrap_used = "deny"
expect_used = "deny"
```

### Coverage Configuration

Environment variables for coverage collection:

```bash
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-C instrument-coverage"
export LLVM_PROFILE_FILE="cargo-test-%p-%m.profraw"
```

This comprehensive quality gate system ensures:
- ✅ No incomplete/stub code in production
- ✅ High test coverage for critical components
- ✅ Consistent code quality standards
- ✅ Early detection of quality issues
- ✅ Actionable feedback for improvements