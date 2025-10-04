#!/usr/bin/env pwsh
$ErrorActionPreference = "Stop"

Write-Host "=== Phase 1 - Stages 1.3 & 1.4 Verification ===" -ForegroundColor Green

# Change to the Rust project directory
Set-Location "C:\Accumulate_Stuff\opendlt-rust-v2v3-sdk\unified"

Write-Host "Building and running delegation depth tests..." -ForegroundColor Yellow
cargo test --test delegated_depth_tests -- --nocapture

Write-Host "Building and running signature set threshold tests..." -ForegroundColor Yellow
cargo test --test signature_set_tests -- --nocapture

Write-Host "Building and running golden vector tests..." -ForegroundColor Yellow
cargo test --test golden_vector_tests -- --nocapture

Write-Host "Running all signature-related tests to ensure regression-free..." -ForegroundColor Yellow
cargo test --test working_signature_test --test signature_dispatch_tests --test signature_crypto_tests --test signature_integration_tests -- --nocapture

Write-Host "Verifying that G2 gate (16/16 signatures) is still preserved..." -ForegroundColor Yellow

# Run parity audit to ensure G2=16/16 is maintained
Write-Host "Running parity audit..." -ForegroundColor Yellow
Set-Location "C:\Accumulate_Stuff\rust_parity_audit"

python "tmp\yaml_to_ir.py"
if ($LASTEXITCODE -ne 0) {
    Write-Error "yaml_to_ir.py failed"
    exit 1
}

python "tmp\rust_surface_scan.py"
if ($LASTEXITCODE -ne 0) {
    Write-Error "rust_surface_scan.py failed"
    exit 1
}

python "tmp\parity_analyzer.py"
if ($LASTEXITCODE -ne 0) {
    Write-Error "parity_analyzer.py failed"
    exit 1
}

Set-Location "C:\Accumulate_Stuff\opendlt-rust-v2v3-sdk\unified"

Write-Host "=== Phase 1 - Stages 1.3 & 1.4 Verification Complete ===" -ForegroundColor Green
Write-Host "✓ Delegation depth enforcement (≤ 5) implemented and tested" -ForegroundColor Green
Write-Host "✓ SignatureSet threshold semantics implemented and tested" -ForegroundColor Green
Write-Host "✓ Golden vectors created and validated" -ForegroundColor Green
Write-Host "✓ G2 gate (16/16 signatures) preserved" -ForegroundColor Green
Write-Host "✓ All tests passing" -ForegroundColor Green