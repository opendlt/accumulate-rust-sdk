#!/bin/bash
# Development environment setup script for Accumulate Rust SDK

set -e

echo "🔧 Setting up Accumulate Rust SDK development environment..."

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed!"
    echo "💡 Install Rust: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"
    exit 1
fi

echo "✅ Rust toolchain detected"

# Install required components
echo "📦 Installing Rust components..."
rustup component add rustfmt clippy

# Install development tools
echo "🔧 Installing development tools..."
cargo install cargo-llvm-cov
cargo install cargo-audit
cargo install cargo-outdated

# Set up git hooks (optional)
if [ -d ".git" ]; then
    echo "🪝 Setting up git pre-commit hook..."
    cp scripts/pre-commit.sh .git/hooks/pre-commit
    chmod +x .git/hooks/pre-commit
    echo "✅ Pre-commit hook installed"
fi

# Run initial checks
echo "🧪 Running initial checks..."
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features

# Generate documentation
echo "📚 Building documentation..."
cargo doc --all-features --no-deps

# Run DevNet discovery
echo "🔍 Running DevNet discovery..."
if cargo run --bin devnet_discovery; then
    echo "✅ DevNet configuration discovered"
else
    echo "⚠️  DevNet not available (this is OK for development)"
fi

echo ""
echo "🎉 Development environment setup complete!"
echo ""
echo "📋 Available commands:"
echo "  make help          - Show all available commands"
echo "  make dev           - Run development checks (fmt, lint, test)"
echo "  make ci-check      - Run full CI checks locally"
echo "  make coverage      - Generate coverage report"
echo "  make examples      - Run example programs"
echo ""
echo "🚀 Happy coding!"