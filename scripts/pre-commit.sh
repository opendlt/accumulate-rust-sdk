#!/bin/bash
# Pre-commit hook for Accumulate Rust SDK
# Run this before committing to ensure code quality

set -e

echo "🔍 Running pre-commit checks..."

# Check formatting
echo "🎨 Checking code formatting..."
if ! cargo fmt --all -- --check; then
    echo "❌ Code formatting check failed!"
    echo "💡 Run: cargo fmt --all"
    exit 1
fi

# Run clippy
echo "📎 Running clippy lints..."
if ! cargo clippy --all-targets --all-features -- -D warnings; then
    echo "❌ Clippy check failed!"
    echo "💡 Fix the warnings above"
    exit 1
fi

# Run tests
echo "🧪 Running tests..."
if ! cargo test --all-features; then
    echo "❌ Tests failed!"
    echo "💡 Fix failing tests"
    exit 1
fi

# Run conformance tests
echo "🧪 Running conformance tests..."
if ! cargo test conformance --all-features; then
    echo "❌ Conformance tests failed!"
    echo "💡 Check TypeScript SDK parity"
    exit 1
fi

# Check package
echo "📦 Checking package..."
if ! cargo package --allow-dirty > /dev/null 2>&1; then
    echo "❌ Package check failed!"
    echo "💡 Fix packaging issues"
    exit 1
fi

echo "✅ All pre-commit checks passed!"
echo "🚀 Ready to commit!"