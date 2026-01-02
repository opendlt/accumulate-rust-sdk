# Accumulate Rust SDK Development Commands

.PHONY: help fmt lint test coverage clean install-tools ci-check package release

# Default target
help:
	@echo "Accumulate Rust SDK Development Commands"
	@echo "========================================"
	@echo ""
	@echo "Development:"
	@echo "  make fmt          - Format code with rustfmt"
	@echo "  make lint         - Run clippy lints"
	@echo "  make test         - Run all tests"
	@echo "  make coverage     - Generate test coverage report"
	@echo ""
	@echo "CI/Production:"
	@echo "  make ci-check     - Run all CI checks (fmt, lint, test)"
	@echo "  make package      - Test packaging for crates.io"
	@echo "  make release      - Build release artifacts"
	@echo ""
	@echo "Setup:"
	@echo "  make install-tools - Install required development tools"
	@echo "  make clean        - Clean build artifacts"

# Install development tools
install-tools:
	@echo "🔧 Installing development tools..."
	cargo install cargo-llvm-cov
	cargo install cargo-audit
	cargo install cargo-outdated
	rustup component add rustfmt clippy

# Format code
fmt:
	@echo "Formatting code..."
	cargo fmt --all

# Check formatting
fmt-check:
	@echo "Checking code formatting..."
	cargo fmt --all -- --check

# Run clippy lints
lint:
	@echo "Running clippy lints..."
	cargo clippy --all-targets --all-features -- -D warnings

# Run tests
test:
	@echo "Running tests..."
	cargo test --all-features

# Run tests with specific patterns
test-unit:
	@echo "Running unit tests..."
	cargo test --lib --all-features

test-integration:
	@echo "Running integration tests..."
	cargo test --test integration --all-features

test-conformance:
	@echo "Running conformance tests..."
	cargo test conformance --all-features

# Generate coverage report
coverage:
	@echo "Generating coverage report..."
	cargo llvm-cov --all-features --lcov --output-path lcov.info
	cargo llvm-cov --all-features --html

# Coverage with minimum threshold
coverage-gate:
	@echo "Running coverage with 85% threshold..."
	cargo llvm-cov --all-features --fail-under-lines 85

# Security audit
audit:
	@echo "Running security audit..."
	cargo audit

# Check for outdated dependencies
outdated:
	@echo "Checking for outdated dependencies..."
	cargo outdated

# Full CI check (run locally before pushing)
ci-check: fmt-check lint test coverage-gate audit
	@echo "All CI checks passed!"

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	rm -f lcov.info
	rm -rf target/llvm-cov-target

# Test packaging
package:
	@echo "Testing package creation..."
	cargo package --allow-dirty

# Dry run publish
package-check:
	@echo "Testing package publication (dry run)..."
	cargo publish --dry-run

# Build release artifacts
release:
	@echo "Building release artifacts..."
	cargo build --release --all-features

# Run DevNet discovery
discover:
	@echo "Discovering DevNet configuration..."
	cargo run --bin devnet_discovery

# Run examples
examples:
	@echo "Running key examples..."
	cargo run --example 100_keygen_lite_urls
	cargo run --example 120_faucet_local_devnet
	cargo run --example 999_zero_to_hero

# Benchmark tests (if any)
bench:
	@echo "Running benchmarks..."
	cargo test --benches --all-features

# Documentation
docs:
	@echo "Building documentation..."
	cargo doc --all-features --no-deps

# Documentation with private items
docs-private:
	@echo "Building documentation (including private)..."
	cargo doc --all-features --document-private-items --no-deps

# Open documentation in browser
docs-open:
	@echo "Opening documentation..."
	cargo doc --all-features --no-deps --open

# Check compilation without running tests
check:
	@echo "Checking compilation..."
	cargo check --all-targets --all-features

# Development workflow (format, lint, test)
dev: fmt lint test
	@echo "Development checks complete!"

# Pre-commit hook simulation
pre-commit: fmt-check lint test-unit
	@echo "Pre-commit checks passed!"