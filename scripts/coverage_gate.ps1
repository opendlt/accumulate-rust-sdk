#!/usr/bin/env pwsh
# Coverage Gate Script for Accumulate Rust SDK
# Enforces minimum coverage thresholds for different code areas

param(
    [Parameter(HelpMessage="Overall coverage threshold (default: 70%)")]
    [int]$OverallThreshold = 70,

    [Parameter(HelpMessage="Critical code coverage threshold (default: 85%)")]
    [int]$CriticalThreshold = 85,

    [Parameter(HelpMessage="Skip coverage generation and use existing lcov.info")]
    [switch]$SkipGeneration,

    [Parameter(HelpMessage="Verbose output")]
    [switch]$Verbose
)

# ANSI colors for output
$Red = "`e[31m"
$Green = "`e[32m"
$Yellow = "`e[33m"
$Blue = "`e[34m"
$Magenta = "`e[35m"
$Cyan = "`e[36m"
$Reset = "`e[0m"

function Write-Header {
    param($Message)
    Write-Host "${Cyan}================================================================${Reset}"
    Write-Host "${Cyan} $Message${Reset}"
    Write-Host "${Cyan}================================================================${Reset}"
}

function Write-Success {
    param($Message)
    Write-Host "${Green}✅ $Message${Reset}"
}

function Write-Warning {
    param($Message)
    Write-Host "${Yellow}⚠️  $Message${Reset}"
}

function Write-Error {
    param($Message)
    Write-Host "${Red}❌ $Message${Reset}"
}

function Write-Info {
    param($Message)
    Write-Host "${Blue}ℹ️  $Message${Reset}"
}

function Write-Progress {
    param($Message)
    Write-Host "${Magenta}🔄 $Message${Reset}"
}

Write-Header "Accumulate Rust SDK Coverage Gate"

# Check if cargo-llvm-cov is installed
Write-Progress "Checking cargo-llvm-cov installation..."
$llvmCovCheck = & cargo llvm-cov --version 2>$null
if ($LASTEXITCODE -ne 0) {
    Write-Error "cargo-llvm-cov not found. Installing..."
    & cargo install cargo-llvm-cov --locked
    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to install cargo-llvm-cov"
        exit 1
    }
    Write-Success "cargo-llvm-cov installed successfully"
} else {
    Write-Success "cargo-llvm-cov is available: $($llvmCovCheck.Split("`n")[0])"
}

# Generate coverage report
if (-not $SkipGeneration) {
    Write-Progress "Generating coverage report..."

    # Clean previous coverage data
    if (Test-Path "target/lcov.info") {
        Remove-Item "target/lcov.info" -Force
    }

    # Generate LCOV report
    Write-Info "Running: cargo llvm-cov --all-features --lcov --output-path target/lcov.info"
    & cargo llvm-cov --all-features --lcov --output-path target/lcov.info

    if ($LASTEXITCODE -ne 0) {
        Write-Error "Failed to generate coverage report"
        exit 1
    }

    Write-Success "Coverage report generated: target/lcov.info"
} else {
    Write-Info "Skipping coverage generation, using existing lcov.info"
}

# Check if lcov.info exists
$lcovPath = "target/lcov.info"
if (-not (Test-Path $lcovPath)) {
    Write-Error "Coverage file not found: $lcovPath"
    exit 1
}

Write-Progress "Parsing coverage data..."

# Parse LCOV file
$lcovContent = Get-Content $lcovPath
$sourceFiles = @{}
$currentFile = ""

foreach ($line in $lcovContent) {
    if ($line.StartsWith("SF:")) {
        $currentFile = $line.Substring(3).Replace("/", "\")
        if (-not $sourceFiles.ContainsKey($currentFile)) {
            $sourceFiles[$currentFile] = @{
                LinesFound = 0
                LinesHit = 0
                Coverage = 0.0
            }
        }
    } elseif ($line.StartsWith("LF:")) {
        $sourceFiles[$currentFile].LinesFound = [int]($line.Substring(3))
    } elseif ($line.StartsWith("LH:")) {
        $sourceFiles[$currentFile].LinesHit = [int]($line.Substring(3))
        if ($sourceFiles[$currentFile].LinesFound -gt 0) {
            $sourceFiles[$currentFile].Coverage = ($sourceFiles[$currentFile].LinesHit / $sourceFiles[$currentFile].LinesFound) * 100
        }
    }
}

# Calculate overall coverage
$totalLinesFound = ($sourceFiles.Values | Measure-Object -Property LinesFound -Sum).Sum
$totalLinesHit = ($sourceFiles.Values | Measure-Object -Property LinesHit -Sum).Sum
$overallCoverage = if ($totalLinesFound -gt 0) { ($totalLinesHit / $totalLinesFound) * 100 } else { 0.0 }

# Define critical file patterns
$criticalPatterns = @(
    "*\src\codec\*",
    "*\src\crypto\*",
    "*\src\canonjson.rs"
)

# Calculate critical coverage
$criticalFiles = @()
foreach ($file in $sourceFiles.Keys) {
    foreach ($pattern in $criticalPatterns) {
        if ($file -like $pattern) {
            $criticalFiles += $file
            break
        }
    }
}

$criticalLinesFound = 0
$criticalLinesHit = 0
foreach ($file in $criticalFiles) {
    $criticalLinesFound += $sourceFiles[$file].LinesFound
    $criticalLinesHit += $sourceFiles[$file].LinesHit
}

$criticalCoverage = if ($criticalLinesFound -gt 0) { ($criticalLinesHit / $criticalLinesFound) * 100 } else { 100.0 }

# Display results
Write-Header "Coverage Summary"

Write-Host ""
Write-Host "${Blue}📊 Overall Coverage:${Reset}"
Write-Host "   Lines Total: $totalLinesFound"
Write-Host "   Lines Hit:   $totalLinesHit"
if ($overallCoverage -ge $OverallThreshold) {
    Write-Host "   Coverage:    ${Green}$($overallCoverage.ToString("F1"))%${Reset} (threshold: $OverallThreshold%)"
} else {
    Write-Host "   Coverage:    ${Red}$($overallCoverage.ToString("F1"))%${Reset} (threshold: $OverallThreshold%)"
}

Write-Host ""
Write-Host "${Blue}🔒 Critical Code Coverage:${Reset}"
Write-Host "   Lines Total: $criticalLinesFound"
Write-Host "   Lines Hit:   $criticalLinesHit"
if ($criticalCoverage -ge $CriticalThreshold) {
    Write-Host "   Coverage:    ${Green}$($criticalCoverage.ToString("F1"))%${Reset} (threshold: $CriticalThreshold%)"
} else {
    Write-Host "   Coverage:    ${Red}$($criticalCoverage.ToString("F1"))%${Reset} (threshold: $CriticalThreshold%)"
}

# Show critical files
if ($criticalFiles.Count -gt 0) {
    Write-Host ""
    Write-Host "${Blue}📋 Critical Files:${Reset}"
    foreach ($file in $criticalFiles | Sort-Object) {
        $coverage = $sourceFiles[$file].Coverage
        $status = if ($coverage -ge $CriticalThreshold) { "${Green}✅${Reset}" } else { "${Red}❌${Reset}" }
        Write-Host "   $status $($coverage.ToString("F1").PadLeft(5))% $file"
    }
}

# Show low coverage files (< 50%)
$lowCoverageFiles = $sourceFiles.GetEnumerator() | Where-Object { $_.Value.Coverage -lt 50 -and $_.Value.LinesFound -gt 5 } | Sort-Object { $_.Value.Coverage }
if ($lowCoverageFiles.Count -gt 0) {
    Write-Host ""
    Write-Host "${Yellow}⚠️  Low Coverage Files (< 50%):${Reset}"
    foreach ($file in $lowCoverageFiles | Select-Object -First 10) {
        $coverage = $file.Value.Coverage
        Write-Host "   ${Red}❌${Reset} $($coverage.ToString("F1").PadLeft(5))% $($file.Key)"
    }
    if ($lowCoverageFiles.Count -gt 10) {
        Write-Host "   ... and $($lowCoverageFiles.Count - 10) more files"
    }
}

# Check thresholds
$overallPassed = $overallCoverage -ge $OverallThreshold
$criticalPassed = $criticalCoverage -ge $CriticalThreshold

Write-Host ""
Write-Header "Coverage Gate Results"

if ($overallPassed) {
    Write-Success "Overall coverage passed: $($overallCoverage.ToString("F1"))% >= $OverallThreshold%"
} else {
    Write-Error "Overall coverage failed: $($overallCoverage.ToString("F1"))% < $OverallThreshold%"
}

if ($criticalPassed) {
    Write-Success "Critical coverage passed: $($criticalCoverage.ToString("F1"))% >= $CriticalThreshold%"
} else {
    Write-Error "Critical coverage failed: $($criticalCoverage.ToString("F1"))% < $CriticalThreshold%"
}

# Final result
if ($overallPassed -and $criticalPassed) {
    Write-Host ""
    Write-Success "🎉 All coverage gates passed!"
    Write-Info "HTML report: target/llvm-cov/html/index.html"
    Write-Info "LCOV report: target/lcov.info"
    exit 0
} else {
    Write-Host ""
    Write-Error "💥 Coverage gates failed!"
    Write-Info "Improve test coverage for the files listed above"
    Write-Info "HTML report: target/llvm-cov/html/index.html"
    Write-Info "LCOV report: target/lcov.info"
    exit 1
}