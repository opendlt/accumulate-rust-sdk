#!/usr/bin/env bash
set -euo pipefail
echo "=== Rust SDK Parity Gate (LOCAL) ==="

# Environment setup
export AUDIT_DIR="C:/Accumulate_Stuff/rust_parity_audit"

echo "Running audit pipeline..."

# 1) Run audit pipeline
python "$AUDIT_DIR/tmp/yaml_to_ir.py"
echo "YAML to IR conversion completed"

python "$AUDIT_DIR/tmp/rust_surface_scan.py"
echo "Rust surface scan completed"

python "$AUDIT_DIR/tmp/parity_analyzer.py"
echo "Parity analysis completed"

# 2) Verify final report includes FULL PARITY
report="$AUDIT_DIR/reports/RUST_vs_Go_Parity_Report.md"
if [ ! -f "$report" ]; then
  echo "Parity report not found: $report"
  exit 1
fi

echo "Checking parity report..."
head -80 "$report"

if ! grep -q "STATUS=FULL PARITY" "$report"; then
  echo "PARITY GATE FAILED"
  exit 1
fi

echo "PARITY GATE PASSED"