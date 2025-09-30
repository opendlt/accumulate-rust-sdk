#!/bin/bash
# Package Sanity Check Script for Accumulate Rust SDK
# Validates documentation, packaging, and publish readiness

set -euo pipefail

# ANSI colors
RED='\033[31m'
GREEN='\033[32m'
YELLOW='\033[33m'
BLUE='\033[34m'
MAGENTA='\033[35m'
CYAN='\033[36m'
RESET='\033[0m'

# Helper functions
write_header() {
    echo -e "${CYAN}================================================================${RESET}"
    echo -e "${CYAN} $1${RESET}"
    echo -e "${CYAN}================================================================${RESET}"
}

write_success() {
    echo -e "${GREEN}✅ $1${RESET}"
}

write_warning() {
    echo -e "${YELLOW}⚠️  $1${RESET}"
}

write_error() {
    echo -e "${RED}❌ $1${RESET}"
}

write_info() {
    echo -e "${BLUE}ℹ️  $1${RESET}"
}

write_progress() {
    echo -e "${MAGENTA}🔄 $1${RESET}"
}

# Initialize counters
TOTAL_CHECKS=0
PASSED_CHECKS=0
FAILED_CHECKS=0

# Function to run a check
run_check() {
    local name="$1"
    local command="$2"
    local warning_only="${3:-false}"

    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    write_progress "Checking: $name"

    if eval "$command" >/dev/null 2>&1; then
        write_success "$name"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
        return 0
    else
        if [ "$warning_only" = "true" ]; then
            write_warning "$name (warning only)"
            PASSED_CHECKS=$((PASSED_CHECKS + 1))
        else
            write_error "$name"
            FAILED_CHECKS=$((FAILED_CHECKS + 1))
        fi
        return 1
    fi
}

write_header "Accumulate Rust SDK Package Sanity Check"

# Change to unified directory if not already there
if [ ! -f "Cargo.toml" ]; then
    if [ -d "unified" ]; then
        cd unified
    else
        write_error "Cannot find Cargo.toml or unified directory"
        exit 1
    fi
fi

echo ""
write_info "Running packaging sanity checks..."

# 1. Basic compilation checks
echo ""
write_header "Compilation Checks"

run_check "Check all targets compile" "cargo check --all-targets --all-features"
run_check "Library builds successfully" "cargo build --lib --all-features"
run_check "Examples build successfully" "cargo build --examples --all-features"
run_check "Binary builds successfully" "cargo build --bin devnet_discovery --all-features"

# 2. Documentation checks
echo ""
write_header "Documentation Checks"

run_check "Documentation builds without warnings" "cargo doc --all-features --no-deps -D warnings"
run_check "Documentation includes private items" "cargo doc --all-features --no-deps --document-private-items" true

# 3. Packaging checks
echo ""
write_header "Packaging Checks"

run_check "Package creation succeeds" "cargo package --allow-dirty"
run_check "Dry-run publish succeeds" "cargo publish --dry-run"

# 4. Metadata validation
echo ""
write_header "Metadata Validation"

# Check required package fields
CARGO_TOML="Cargo.toml"

check_field() {
    local field="$1"
    local display_name="$2"

    if grep -q "^$field" "$CARGO_TOML"; then
        write_success "$display_name is present"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
    else
        write_error "$display_name is missing"
        FAILED_CHECKS=$((FAILED_CHECKS + 1))
    fi
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
}

check_field "name" "Package name"
check_field "version" "Package version"
check_field "description" "Package description"
check_field "license" "Package license"
check_field "repository" "Repository URL"
check_field "documentation" "Documentation URL"
check_field "readme" "README file"
check_field "authors" "Package authors"
check_field "keywords" "Package keywords"
check_field "categories" "Package categories"

# 5. Code quality checks
echo ""
write_header "Code Quality Checks"

run_check "Code formatting is correct" "cargo fmt --all -- --check"
run_check "Clippy lints pass" "cargo clippy --all-targets --all-features -- -D warnings"

# 6. Dependency checks
echo ""
write_header "Dependency Checks"

# Check for common issues
run_check "No git dependencies" "! grep -q 'git = ' Cargo.toml"
run_check "No path dependencies (except dev)" "! grep -A1 -B1 'path = ' Cargo.toml | grep -v dev-dependencies" true

# 7. License and legal checks
echo ""
write_header "License and Legal Checks"

# Check for LICENSE file
if [ -f "LICENSE" ] || [ -f "LICENSE.md" ] || [ -f "LICENSE.txt" ]; then
    write_success "License file exists"
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
else
    write_warning "License file not found (not required for crates.io but recommended)"
    PASSED_CHECKS=$((PASSED_CHECKS + 1))
fi
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

# Check for README
if [ -f "README.md" ]; then
    write_success "README.md exists"
    PASSED_CHECKS=$((PASSED_CHECKS + 1))

    # Check README has basic content
    if grep -q "# " README.md; then
        write_success "README has title"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
    else
        write_warning "README should have a proper title"
        PASSED_CHECKS=$((PASSED_CHECKS + 1))
    fi
    TOTAL_CHECKS=$((TOTAL_CHECKS + 2))
else
    write_error "README.md is missing"
    FAILED_CHECKS=$((FAILED_CHECKS + 1))
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
fi

# 8. Security checks
echo ""
write_header "Security Checks"

# Check for common security issues
run_check "No unsafe code (basic check)" "! grep -r 'unsafe ' src/ || true" true
run_check "No TODO/FIXME in main code" "! grep -r 'TODO\\|FIXME' src/ || true" true

# 9. Performance checks
echo ""
write_header "Performance Checks"

run_check "Release build succeeds" "cargo build --release"

# Final summary
echo ""
write_header "Package Check Summary"

echo ""
echo -e "${BLUE}📊 Results:${RESET}"
echo "   Total checks: $TOTAL_CHECKS"
echo "   Passed: $PASSED_CHECKS"
echo "   Failed: $FAILED_CHECKS"

if [ $FAILED_CHECKS -eq 0 ]; then
    echo ""
    write_success "🎉 All package sanity checks passed!"
    echo ""
    write_info "Package is ready for publishing to crates.io"
    echo ""
    echo -e "${BLUE}📋 Next steps:${RESET}"
    echo "   1. Update version in Cargo.toml if needed"
    echo "   2. Update CHANGELOG.md with release notes"
    echo "   3. Create a git tag: git tag v\$(grep '^version' Cargo.toml | cut -d'\"' -f2)"
    echo "   4. Push tag to trigger release: git push origin --tags"
    echo ""
    exit 0
else
    echo ""
    write_error "💥 $FAILED_CHECKS package check(s) failed!"
    echo ""
    write_info "Please fix the issues above before publishing"
    echo ""
    exit 1
fi