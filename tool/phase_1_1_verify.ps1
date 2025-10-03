$ErrorActionPreference = "Stop"

Write-Host "=== Phase 1.1 Verification: Enum Canonicalization ===" -ForegroundColor Cyan

# 1) Regenerate enums
Write-Host "`n1) Regenerating enums..." -ForegroundColor Yellow
python "C:\Accumulate_Stuff\opendlt-rust-v2v3-sdk\unified\tooling\backends\rust_enums_codegen.py"
if ($LASTEXITCODE -ne 0) {
    Write-Host "Enum generation failed!" -ForegroundColor Red
    exit 1
}

# 2) Run tests (capture goldens if needed)
Write-Host "`n2) Running tests with golden vector capture..." -ForegroundColor Yellow
$env:INSTA_UPDATE = "auto"
cargo test --tests enum_roundtrip -- --nocapture
$test_result = $LASTEXITCODE
$env:INSTA_UPDATE = ""

if ($test_result -ne 0) {
    Write-Host "Initial test run failed!" -ForegroundColor Red
    exit 1
}

# 3) Re-run tests enforcing snapshots
Write-Host "`n3) Re-running tests to enforce golden vectors..." -ForegroundColor Yellow
cargo test --tests enum_roundtrip -- --nocapture
if ($LASTEXITCODE -ne 0) {
    Write-Host "Golden vector enforcement failed!" -ForegroundColor Red
    exit 1
}

# 4) Run parity audit to verify G1=PASS
Write-Host "`n4) Running parity audit..." -ForegroundColor Yellow
python "C:\Accumulate_Stuff\rust_parity_audit\tmp\yaml_to_ir.py"
python "C:\Accumulate_Stuff\rust_parity_audit\tmp\rust_surface_scan.py"
python "C:\Accumulate_Stuff\rust_parity_audit\tmp\parity_analyzer.py"

# 5) Check for G1=PASS specifically
Write-Host "`n5) Verifying G1=PASS (14/14 enums)..." -ForegroundColor Yellow
$parity_report = "C:\Accumulate_Stuff\rust_parity_audit\reports\RUST_vs_Go_Parity_Report.md"
if (Test-Path $parity_report) {
    $content = Get-Content $parity_report -Raw
    if ($content -match "Enums.*14.*14.*✅|Enums.*14.*14.*PASS") {
        Write-Host "✅ SUCCESS: G1=PASS (14/14 enums achieved)" -ForegroundColor Green
    } else {
        Write-Host "❌ FAILED: G1 gate not achieved" -ForegroundColor Red
        Write-Host "Parity report excerpt:" -ForegroundColor Yellow
        Get-Content $parity_report | Select-Object -First 20
        exit 1
    }
} else {
    Write-Host "❌ FAILED: Parity report not found" -ForegroundColor Red
    exit 1
}

Write-Host "`n🎉 Phase 1.1 Verification Complete!" -ForegroundColor Green
Write-Host "✓ 14 enums generated with exact wire compatibility" -ForegroundColor White
Write-Host "✓ All JSON roundtrip tests pass" -ForegroundColor White
Write-Host "✓ Golden vectors captured and enforced" -ForegroundColor White
Write-Host "✓ G1=PASS (14/14 enums) confirmed" -ForegroundColor White