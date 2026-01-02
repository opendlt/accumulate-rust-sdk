$ErrorActionPreference = "Stop"
Write-Host "=== Rust SDK Parity Gate (LOCAL) ==="

# Environment setup
$env:AUDIT_DIR = "C:\Accumulate_Stuff\rust_parity_audit"

Write-Host "Running audit pipeline..."

# 1) Run audit pipeline
try {
    python "$env:AUDIT_DIR\tmp\yaml_to_ir.py"
    Write-Host "YAML to IR conversion completed"
} catch {
    Write-Error "YAML to IR conversion failed: $_"
}

try {
    python "$env:AUDIT_DIR\tmp\rust_surface_scan.py"
    Write-Host "Rust surface scan completed"
} catch {
    Write-Error "Rust surface scan failed: $_"
}

try {
    python "$env:AUDIT_DIR\tmp\parity_analyzer.py"
    Write-Host "Parity analysis completed"
} catch {
    Write-Error "Parity analysis failed: $_"
}

# 2) Verify final report includes FULL PARITY
$report = Join-Path $env:AUDIT_DIR "reports\RUST_vs_Go_Parity_Report.md"
if (!(Test-Path $report)) {
  Write-Error "Parity report not found: $report"
}

Write-Host "Checking parity report..."
$head = (Get-Content $report -TotalCount 80) -join "`n"
$head | Write-Host

if ($head -notmatch "STATUS=FULL PARITY") {
  Write-Error "PARITY GATE FAILED (see report head above)"
}

Write-Host "PARITY GATE PASSED"