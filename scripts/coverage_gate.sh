#!/bin/bash
# Coverage Gate Script for Accumulate Rust SDK
# Enforces minimum coverage thresholds for different code areas

set -euo pipefail

# Default thresholds
OVERALL_THRESHOLD=70
CRITICAL_THRESHOLD=85
SKIP_GENERATION=false
VERBOSE=false

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
    echo -e "${GREEN} $1${RESET}"
}

write_warning() {
    echo -e "${YELLOW}  $1${RESET}"
}

write_error() {
    echo -e "${RED} $1${RESET}"
}

write_info() {
    echo -e "${BLUE}ℹ  $1${RESET}"
}

write_progress() {
    echo -e "${MAGENTA} $1${RESET}"
}

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --overall-threshold)
            OVERALL_THRESHOLD="$2"
            shift 2
            ;;
        --critical-threshold)
            CRITICAL_THRESHOLD="$2"
            shift 2
            ;;
        --skip-generation)
            SKIP_GENERATION=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo "Options:"
            echo "  --overall-threshold N   Overall coverage threshold (default: 70)"
            echo "  --critical-threshold N  Critical code coverage threshold (default: 85)"
            echo "  --skip-generation       Skip coverage generation, use existing lcov.info"
            echo "  --verbose               Verbose output"
            echo "  --help                  Show this help"
            exit 0
            ;;
        *)
            write_error "Unknown option: $1"
            exit 1
            ;;
    esac
done

write_header "Accumulate Rust SDK Coverage Gate"

# Check if cargo-llvm-cov is installed
write_progress "Checking cargo-llvm-cov installation..."
if ! cargo llvm-cov --version >/dev/null 2>&1; then
    write_error "cargo-llvm-cov not found. Installing..."
    cargo install cargo-llvm-cov --locked
    write_success "cargo-llvm-cov installed successfully"
else
    write_success "cargo-llvm-cov is available: $(cargo llvm-cov --version | head -n1)"
fi

# Generate coverage report
if [ "$SKIP_GENERATION" = false ]; then
    write_progress "Generating coverage report..."

    # Clean previous coverage data
    rm -f target/lcov.info

    # Generate LCOV report
    write_info "Running: cargo llvm-cov --all-features --lcov --output-path target/lcov.info"
    cargo llvm-cov --all-features --lcov --output-path target/lcov.info

    write_success "Coverage report generated: target/lcov.info"
else
    write_info "Skipping coverage generation, using existing lcov.info"
fi

# Check if lcov.info exists
LCOV_PATH="target/lcov.info"
if [ ! -f "$LCOV_PATH" ]; then
    write_error "Coverage file not found: $LCOV_PATH"
    exit 1
fi

write_progress "Parsing coverage data..."

# Create temporary files for processing
TEMP_DIR=$(mktemp -d)
trap "rm -rf $TEMP_DIR" EXIT

# Extract coverage data
awk '
BEGIN { current_file = "" }
/^SF:/ { current_file = substr($0, 4); gsub(/\//, "\\", current_file) }
/^LF:/ && current_file != "" { lines_found[current_file] = substr($0, 4) }
/^LH:/ && current_file != "" {
    lines_hit[current_file] = substr($0, 4)
    if (lines_found[current_file] > 0) {
        coverage[current_file] = (lines_hit[current_file] / lines_found[current_file]) * 100
    } else {
        coverage[current_file] = 0
    }
}
END {
    total_found = 0
    total_hit = 0

    for (file in lines_found) {
        total_found += lines_found[file]
        total_hit += lines_hit[file]
        print file ":" lines_found[file] ":" lines_hit[file] ":" coverage[file]
    }

    overall = (total_found > 0) ? (total_hit / total_found) * 100 : 0
    print "OVERALL:" total_found ":" total_hit ":" overall
}
' "$LCOV_PATH" > "$TEMP_DIR/coverage_data"

# Read overall coverage
OVERALL_LINE=$(grep "^OVERALL:" "$TEMP_DIR/coverage_data")
OVERALL_TOTAL=$(echo "$OVERALL_LINE" | cut -d: -f2)
OVERALL_HIT=$(echo "$OVERALL_LINE" | cut -d: -f3)
OVERALL_COVERAGE=$(echo "$OVERALL_LINE" | cut -d: -f4)

# Calculate critical coverage
CRITICAL_PATTERNS=(
    "*/src/codec/*"
    "*/src/crypto/*"
    "*/src/canonjson.rs"
)

CRITICAL_TOTAL=0
CRITICAL_HIT=0
CRITICAL_FILES=()

while IFS=: read -r file found hit coverage; do
    if [ "$file" = "OVERALL" ]; then
        continue
    fi

    for pattern in "${CRITICAL_PATTERNS[@]}"; do
        if [[ "$file" == $pattern ]]; then
            CRITICAL_TOTAL=$((CRITICAL_TOTAL + found))
            CRITICAL_HIT=$((CRITICAL_HIT + hit))
            CRITICAL_FILES+=("$file:$coverage")
            break
        fi
    done
done < "$TEMP_DIR/coverage_data"

CRITICAL_COVERAGE=$(awk "BEGIN { if ($CRITICAL_TOTAL > 0) print ($CRITICAL_HIT / $CRITICAL_TOTAL) * 100; else print 100.0 }")

# Display results
write_header "Coverage Summary"

echo ""
echo -e "${BLUE} Overall Coverage:${RESET}"
echo "   Lines Total: $OVERALL_TOTAL"
echo "   Lines Hit:   $OVERALL_HIT"
if (( $(echo "$OVERALL_COVERAGE >= $OVERALL_THRESHOLD" | bc -l) )); then
    printf "   Coverage:    ${GREEN}%.1f%%${RESET} (threshold: %d%%)\n" "$OVERALL_COVERAGE" "$OVERALL_THRESHOLD"
else
    printf "   Coverage:    ${RED}%.1f%%${RESET} (threshold: %d%%)\n" "$OVERALL_COVERAGE" "$OVERALL_THRESHOLD"
fi

echo ""
echo -e "${BLUE} Critical Code Coverage:${RESET}"
echo "   Lines Total: $CRITICAL_TOTAL"
echo "   Lines Hit:   $CRITICAL_HIT"
if (( $(echo "$CRITICAL_COVERAGE >= $CRITICAL_THRESHOLD" | bc -l) )); then
    printf "   Coverage:    ${GREEN}%.1f%%${RESET} (threshold: %d%%)\n" "$CRITICAL_COVERAGE" "$CRITICAL_THRESHOLD"
else
    printf "   Coverage:    ${RED}%.1f%%${RESET} (threshold: %d%%)\n" "$CRITICAL_COVERAGE" "$CRITICAL_THRESHOLD"
fi

# Show critical files
if [ ${#CRITICAL_FILES[@]} -gt 0 ]; then
    echo ""
    echo -e "${BLUE} Critical Files:${RESET}"
    for file_data in "${CRITICAL_FILES[@]}"; do
        file=$(echo "$file_data" | cut -d: -f1)
        coverage=$(echo "$file_data" | cut -d: -f2)
        if (( $(echo "$coverage >= $CRITICAL_THRESHOLD" | bc -l) )); then
            status="${GREEN} ${RESET}"
        else
            status="${RED} ${RESET}"
        fi
        printf "   %s %5.1f%% %s\n" "$status" "$coverage" "$file"
    done
fi

# Show low coverage files (< 50%)
echo ""
echo -e "${YELLOW}  Low Coverage Files (< 50%):${RESET}"
LOW_COUNT=0
while IFS=: read -r file found hit coverage; do
    if [ "$file" = "OVERALL" ]; then
        continue
    fi
    if (( $(echo "$coverage < 50 && $found > 5" | bc -l) )) && [ $LOW_COUNT -lt 10 ]; then
        printf "   ${RED} ${RESET} %5.1f%% %s\n" "$coverage" "$file"
        LOW_COUNT=$((LOW_COUNT + 1))
    fi
done < "$TEMP_DIR/coverage_data"

# Check thresholds
OVERALL_PASSED=$(echo "$OVERALL_COVERAGE >= $OVERALL_THRESHOLD" | bc -l)
CRITICAL_PASSED=$(echo "$CRITICAL_COVERAGE >= $CRITICAL_THRESHOLD" | bc -l)

echo ""
write_header "Coverage Gate Results"

if [ "$OVERALL_PASSED" = "1" ]; then
    write_success "Overall coverage passed: $(printf "%.1f" "$OVERALL_COVERAGE")% >= $OVERALL_THRESHOLD%"
else
    write_error "Overall coverage failed: $(printf "%.1f" "$OVERALL_COVERAGE")% < $OVERALL_THRESHOLD%"
fi

if [ "$CRITICAL_PASSED" = "1" ]; then
    write_success "Critical coverage passed: $(printf "%.1f" "$CRITICAL_COVERAGE")% >= $CRITICAL_THRESHOLD%"
else
    write_error "Critical coverage failed: $(printf "%.1f" "$CRITICAL_COVERAGE")% < $CRITICAL_THRESHOLD%"
fi

# Final result
if [ "$OVERALL_PASSED" = "1" ] && [ "$CRITICAL_PASSED" = "1" ]; then
    echo ""
    write_success " All coverage gates passed!"
    write_info "HTML report: target/llvm-cov/html/index.html"
    write_info "LCOV report: target/lcov.info"
    exit 0
else
    echo ""
    write_error " Coverage gates failed!"
    write_info "Improve test coverage for the files listed above"
    write_info "HTML report: target/llvm-cov/html/index.html"
    write_info "LCOV report: target/lcov.info"
    exit 1
fi