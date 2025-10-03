$ErrorActionPreference = "Stop"

Write-Host "== Enum codegen ==" -ForegroundColor Green

python "C:\Accumulate_Stuff\opendlt-rust-v2v3-sdk\unified\tooling\backends\rust_enums_codegen.py"

if ($LASTEXITCODE -ne 0) {
    Write-Host "Enum generation failed!" -ForegroundColor Red
    exit $LASTEXITCODE
}

Write-Host "Enum generation completed successfully" -ForegroundColor Green