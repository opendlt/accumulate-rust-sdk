#!/usr/bin/env pwsh
# run_parity_gate.ps1
# Comprehensive parity gate for Accumulate Rust SDK
# Ensures binary compatibility, canonical JSON parity, hash alignment, and signature verification with TypeScript SDK

param(
    [int]$FuzzCount = 1000,
    [int]$CoverageThreshold = 70,
    [string]$LogLevel = "info"
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

Write-Host "🚀 ACCUMULATE RUST SDK PARITY GATE" -ForegroundColor Cyan
Write-Host "===================================" -ForegroundColor Cyan
Write-Host ""

# Change to unified directory
Write-Host "📁 Entering unified directory..." -ForegroundColor Yellow
Push-Location "unified"

try {
    # Step 1: Generate TypeScript fixtures and vectors
    Write-Host "🔧 Generating TypeScript fixtures..." -ForegroundColor Blue

    Write-Host "  → Generating standard fixtures..."
    node "tooling\ts-fixture-exporter\export-fixtures.js"
    if ($LASTEXITCODE -ne 0) {
        throw "TypeScript fixture generation failed"
    }

    Write-Host "  → Generating random test vectors (n=$FuzzCount)..."
    $env:TS_FUZZ_N = $FuzzCount
    node "tooling\ts-fixture-exporter\export-random-vectors.js" > "tests\golden\ts_rand_vectors.jsonl"
    if ($LASTEXITCODE -ne 0) {
        throw "TypeScript random vector generation failed"
    }

    $vectorCount = (Get-Content "tests\golden\ts_rand_vectors.jsonl").Count
    Write-Host "  ✅ Generated $vectorCount random test vectors" -ForegroundColor Green

    # Step 2: Code formatting
    Write-Host "🎨 Formatting Rust code..." -ForegroundColor Blue
    cargo fmt --all
    if ($LASTEXITCODE -ne 0) {
        throw "Code formatting failed"
    }
    Write-Host "  ✅ Code formatting complete" -ForegroundColor Green

    # Step 3: Linting
    Write-Host "🔍 Running Clippy linter..." -ForegroundColor Blue
    cargo clippy --all-targets --all-features -- -D warnings
    if ($LASTEXITCODE -ne 0) {
        throw "Clippy linting failed"
    }
    Write-Host "  ✅ Linting passed with no warnings" -ForegroundColor Green

    # Step 4: Quality gates
    Write-Host "🛡️  Running quality gates..." -ForegroundColor Blue

    Write-Host "  → Checking for TODOs and stubs..."
    cargo test --test no_todos test_prohibited_patterns_detection --offline -q
    if ($LASTEXITCODE -ne 0) {
        throw "TODO/stub detection failed"
    }
    Write-Host "  ✅ No prohibited patterns found" -ForegroundColor Green

    # Step 5: Core functionality tests
    Write-Host "🧪 Running core functionality tests..." -ForegroundColor Blue

    Write-Host "  → Testing canonical JSON implementation..."
    cargo test --lib canonjson --offline -q
    if ($LASTEXITCODE -ne 0) {
        throw "Canonical JSON tests failed"
    }

    Write-Host "  → Testing cryptographic functions..."
    cargo test --lib crypto --offline -q
    if ($LASTEXITCODE -ne 0) {
        throw "Cryptographic tests failed"
    }

    Write-Host "  → Testing codec functionality..."
    cargo test --lib codec --offline -q
    if ($LASTEXITCODE -ne 0) {
        throw "Codec tests failed"
    }

    Write-Host "  ✅ Core functionality tests passed" -ForegroundColor Green

    # Step 6: Parity and roundtrip tests
    Write-Host "🔄 Running parity and roundtrip tests..." -ForegroundColor Blue

    Write-Host "  → Testing TypeScript fuzzing roundtrip..."
    cargo test --test ts_fuzz_roundtrip test_fallback_basic_roundtrip --offline -q
    if ($LASTEXITCODE -ne 0) {
        throw "TypeScript fuzz roundtrip tests failed"
    }

    Write-Host "  → Testing type matrix roundtrips..."
    cargo test --test type_matrix_roundtrip --offline -q
    if ($LASTEXITCODE -ne 0) {
        throw "Type matrix roundtrip tests failed"
    }

    Write-Host "  → Verifying type matrix coverage..."
    cargo test --test type_matrix_verification test_type_names_constant --offline -q
    if ($LASTEXITCODE -ne 0) {
        throw "Type matrix verification failed"
    }

    Write-Host "  ✅ All parity and roundtrip tests passed" -ForegroundColor Green

    # Step 7: Full test suite
    Write-Host "🎯 Running complete test suite..." -ForegroundColor Blue
    cargo test --all-features -q
    if ($LASTEXITCODE -ne 0) {
        throw "Complete test suite failed"
    }
    Write-Host "  ✅ All tests passed" -ForegroundColor Green

    # Step 8: Coverage analysis
    Write-Host "📊 Analyzing code coverage..." -ForegroundColor Blue

    # Check if llvm-cov is available
    $llvmCovAvailable = $true
    try {
        cargo llvm-cov --version | Out-Null
    } catch {
        $llvmCovAvailable = $false
    }

    if ($llvmCovAvailable) {
        cargo llvm-cov --all-features --fail-under-lines $CoverageThreshold
        if ($LASTEXITCODE -ne 0) {
            throw "Coverage threshold of $CoverageThreshold% not met"
        }
        Write-Host "  ✅ Coverage threshold of $CoverageThreshold% achieved" -ForegroundColor Green
    } else {
        Write-Host "  ⚠️  llvm-cov not available, skipping coverage analysis" -ForegroundColor Yellow
        Write-Host "    Install with: cargo install cargo-llvm-cov" -ForegroundColor Gray
    }

    # Step 9: Summary report
    Write-Host ""
    Write-Host "📋 PARITY GATE SUMMARY" -ForegroundColor Cyan
    Write-Host "======================" -ForegroundColor Cyan

    $fixtureFiles = Get-ChildItem "tests\golden\*.json" -ErrorAction SilentlyContinue
    $fixtureCount = if ($fixtureFiles) { $fixtureFiles.Count } else { 0 }

    Write-Host "📦 Golden fixtures: $fixtureCount files" -ForegroundColor White
    Write-Host "🎲 Fuzz vectors: $vectorCount envelopes" -ForegroundColor White
    Write-Host "🎯 Test coverage: ${CoverageThreshold}% threshold" -ForegroundColor White

    # Count protocol types from TYPE_NAMES
    $typeMatrixFile = "src\types_matrix.rs"
    if (Test-Path $typeMatrixFile) {
        $typeMatrixContent = Get-Content $typeMatrixFile -Raw
        $typeNamesMatch = [regex]::Match($typeMatrixContent, 'pub const TYPE_NAMES: &\[&str\] = &\[(.*?)\];', [System.Text.RegularExpressions.RegexOptions]::Singleline)
        if ($typeNamesMatch.Success) {
            $typeNamesContent = $typeNamesMatch.Groups[1].Value
            $typeCount = ([regex]::Matches($typeNamesContent, '"[^"]*"')).Count
            Write-Host "🔄 Type matrix: $typeCount protocol types" -ForegroundColor White
        }
    }

    Write-Host ""
    Write-Host "🟢 Parity locked: binary, canonical JSON, hashes, signatures, fuzz roundtrip = OK" -ForegroundColor Green -BackgroundColor DarkGreen
    Write-Host ""
    Write-Host "✅ All parity gates passed successfully!" -ForegroundColor Green
    Write-Host "   The Rust SDK maintains byte-for-byte compatibility with TypeScript SDK" -ForegroundColor Gray

} catch {
    Write-Host ""
    Write-Host "❌ PARITY GATE FAILED" -ForegroundColor Red -BackgroundColor DarkRed
    Write-Host "=====================" -ForegroundColor Red
    Write-Host ""
    Write-Host "Error: $($_.Exception.Message)" -ForegroundColor Red
    Write-Host ""
    Write-Host "💡 Troubleshooting steps:" -ForegroundColor Yellow
    Write-Host "  1. Ensure Node.js is installed and TypeScript fixtures can be generated" -ForegroundColor Gray
    Write-Host "  2. Check that all Rust dependencies are available" -ForegroundColor Gray
    Write-Host "  3. Verify network connectivity for dependency downloads" -ForegroundColor Gray
    Write-Host "  4. Run individual commands manually to isolate the issue" -ForegroundColor Gray

    exit 1
} finally {
    # Always return to original directory
    Pop-Location
}

Write-Host ""
Write-Host "🏁 Parity gate execution complete" -ForegroundColor Cyan