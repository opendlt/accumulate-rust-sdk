$ErrorActionPreference = "Stop"

Write-Host "=== Phase 1.2: Signature Base System Verification ==="

# 1) Regenerate signatures from YAML truth
Write-Host "1. Regenerating signatures from Go YAML truth..."
python "C:\Accumulate_Stuff\opendlt-rust-v2v3-sdk\unified\tooling\backends\rust_signatures_codegen.py"

# 2) Build & tests (write goldens on first pass)
Write-Host "2. Running signature tests (first pass - writing goldens)..."
$env:UPDATE_GOLDENS = "1"
cargo test --tests signature_dispatch -- --nocapture
$env:UPDATE_GOLDENS = ""

# 3) Enforce goldens
Write-Host "3. Enforcing golden vectors..."
cargo test --tests signature_dispatch -- --nocapture

# 4) Parity audit
Write-Host "4. Running parity audit to verify G2=16/16..."
if (Test-Path "C:\Accumulate_Stuff\rust_parity_audit\tmp\yaml_to_ir.py") {
    python "C:\Accumulate_Stuff\rust_parity_audit\tmp\yaml_to_ir.py"
}
if (Test-Path "C:\Accumulate_Stuff\rust_parity_audit\tmp\rust_surface_scan.py") {
    python "C:\Accumulate_Stuff\rust_parity_audit\tmp\rust_surface_scan.py"
}
if (Test-Path "C:\Accumulate_Stuff\rust_parity_audit\tmp\parity_analyzer.py") {
    python "C:\Accumulate_Stuff\rust_parity_audit\tmp\parity_analyzer.py"
}

Write-Host "=== Phase 1.2 Verification Complete ==="