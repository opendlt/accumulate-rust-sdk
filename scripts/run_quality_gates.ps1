#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Run all quality gates for Accumulate Rust SDK

.DESCRIPTION
    This script runs both the no-stubs/TODO checker and coverage gates in sequence.
    It provides a single command to verify all quality requirements.

.PARAMETER SkipTodos
    Skip the TODO/stubs checking test

.PARAMETER SkipCoverage
    Skip the coverage analysis and gates

.PARAMETER CoverageThreshold
    Override default coverage thresholds (default: 70% overall, 85% critical)

.EXAMPLE
    ./scripts/run_quality_gates.ps1

.EXAMPLE
    ./scripts/run_quality_gates.ps1 -SkipTodos

.EXAMPLE
    ./scripts/run_quality_gates.ps1 -CoverageThreshold 75
#>

param(
    [switch]$SkipTodos,
    [switch]$SkipCoverage,
    [int]$CoverageThreshold = 70
)

$ErrorActionPreference = "Stop"

# Color functions
function Write-Success { param($msg) Write-Host "✅ $msg" -ForegroundColor Green }
function Write-Error { param($msg) Write-Host "❌ $msg" -ForegroundColor Red }
function Write-Info { param($msg) Write-Host "ℹ️  $msg" -ForegroundColor Cyan }
function Write-Step { param($msg) Write-Host "🔄 $msg" -ForegroundColor Magenta }

Write-Host ""
Write-Host "🛡️  ACCUMULATE RUST SDK QUALITY GATES" -ForegroundColor Yellow
Write-Host "=======================================" -ForegroundColor Yellow

$startTime = Get-Date
$gatesPassed = 0
$gatesFailed = 0

# Gate 1: No TODOs/Stubs Check
if (-not $SkipTodos) {
    Write-Step "Running Gate 1: No TODOs/Stubs Check"
    try {
        $result = cargo test --test no_todos 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Success "Gate 1 PASSED: No prohibited patterns found"
            $gatesPassed++
        } else {
            Write-Error "Gate 1 FAILED: Prohibited patterns found in source code"
            Write-Host $result -ForegroundColor Red
            $gatesFailed++
        }
    }
    catch {
        Write-Error "Gate 1 ERROR: Failed to run no_todos test: $_"
        $gatesFailed++
    }
} else {
    Write-Info "Gate 1 SKIPPED: No TODOs/Stubs Check"
}

Write-Host ""

# Gate 2: Coverage Gates
if (-not $SkipCoverage) {
    Write-Step "Running Gate 2: Coverage Analysis"
    try {
        $scriptPath = Join-Path $PSScriptRoot "coverage_gate.ps1"

        if (Test-Path $scriptPath) {
            $result = & $scriptPath -OverallThreshold $CoverageThreshold 2>&1
            if ($LASTEXITCODE -eq 0) {
                Write-Success "Gate 2 PASSED: Coverage thresholds met"
                $gatesPassed++
            } else {
                Write-Error "Gate 2 FAILED: Coverage thresholds not met"
                Write-Host $result -ForegroundColor Red
                $gatesFailed++
            }
        } else {
            Write-Error "Gate 2 ERROR: Coverage gate script not found at $scriptPath"
            $gatesFailed++
        }
    }
    catch {
        Write-Error "Gate 2 ERROR: Failed to run coverage gate: $_"
        $gatesFailed++
    }
} else {
    Write-Info "Gate 2 SKIPPED: Coverage Analysis"
}

# Summary
$endTime = Get-Date
$duration = $endTime - $startTime

Write-Host ""
Write-Host "📊 QUALITY GATES SUMMARY" -ForegroundColor Yellow
Write-Host "=========================" -ForegroundColor Yellow
Write-Host "Duration: $($duration.TotalSeconds.ToString('F1')) seconds"
Write-Host "Gates Passed: $gatesPassed" -ForegroundColor Green
Write-Host "Gates Failed: $gatesFailed" -ForegroundColor Red

if ($gatesFailed -eq 0) {
    Write-Host ""
    Write-Success "🎉 ALL QUALITY GATES PASSED!"
    Write-Info "Your code meets all quality requirements:"
    Write-Info "  ✓ No TODOs, FIXMEs, or unimplemented code"
    Write-Info "  ✓ Coverage thresholds met (≥$CoverageThreshold% overall)"
    Write-Info "  ✓ Critical modules have high coverage (≥85%)"
    Write-Host ""
    exit 0
} else {
    Write-Host ""
    Write-Error "💥 QUALITY GATES FAILED!"
    Write-Info "Please fix the issues above before proceeding:"

    if (-not $SkipTodos -and $gatesPassed -lt 1) {
        Write-Info "  🔧 Remove TODO/FIXME/unimplemented patterns"
        Write-Info "     Run: cargo test --test no_todos"
    }

    if (-not $SkipCoverage -and $gatesPassed -lt 2) {
        Write-Info "  📈 Improve test coverage"
        Write-Info "     Run: ./scripts/coverage_gate.ps1 -Verbose"
        Write-Info "     View: cargo llvm-cov --open"
    }

    Write-Host ""
    exit 1
}