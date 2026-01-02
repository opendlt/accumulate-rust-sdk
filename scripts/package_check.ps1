#!/usr/bin/env pwsh
# Package Sanity Check Script for Accumulate Rust SDK
# Validates documentation, packaging, and publish readiness

param(
    [Parameter(HelpMessage="Skip documentation checks")]
    [switch]$SkipDocs,

    [Parameter(HelpMessage="Skip packaging checks")]
    [switch]$SkipPackaging,

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
    Write-Host "${Green} $Message${Reset}"
}

function Write-Warning {
    param($Message)
    Write-Host "${Yellow}  $Message${Reset}"
}

function Write-Error {
    param($Message)
    Write-Host "${Red} $Message${Reset}"
}

function Write-Info {
    param($Message)
    Write-Host "${Blue}ℹ  $Message${Reset}"
}

function Write-Progress {
    param($Message)
    Write-Host "${Magenta} $Message${Reset}"
}

# Initialize counters
$TotalChecks = 0
$PassedChecks = 0
$FailedChecks = 0

function Test-Command {
    param(
        [string]$Name,
        [string]$Command,
        [bool]$WarningOnly = $false
    )

    $script:TotalChecks++
    Write-Progress "Checking: $Name"

    try {
        $output = Invoke-Expression $Command 2>&1
        if ($LASTEXITCODE -eq 0) {
            Write-Success $Name
            $script:PassedChecks++
            return $true
        } else {
            throw "Command failed with exit code $LASTEXITCODE"
        }
    } catch {
        if ($WarningOnly) {
            Write-Warning "$Name (warning only)"
            $script:PassedChecks++
        } else {
            Write-Error $Name
            if ($Verbose) {
                Write-Host "Error details: $($_.Exception.Message)" -ForegroundColor Red
            }
            $script:FailedChecks++
        }
        return $false
    }
}

Write-Header "Accumulate Rust SDK Package Sanity Check"

# Change to unified directory if not already there
if (-not (Test-Path "Cargo.toml")) {
    if (Test-Path "unified") {
        Set-Location "unified"
    } else {
        Write-Error "Cannot find Cargo.toml or unified directory"
        exit 1
    }
}

Write-Info "Running packaging sanity checks..."

# 1. Basic compilation checks
Write-Header "Compilation Checks"

Test-Command "Check all targets compile" "cargo check --all-targets --all-features"
Test-Command "Library builds successfully" "cargo build --lib --all-features"
Test-Command "Examples build successfully" "cargo build --examples --all-features"
Test-Command "Binary builds successfully" "cargo build --bin devnet_discovery --all-features"

# 2. Documentation checks
if (-not $SkipDocs) {
    Write-Header "Documentation Checks"

    Test-Command "Documentation builds without warnings" "cargo doc --all-features --no-deps -D warnings"
    Test-Command "Documentation includes private items" "cargo doc --all-features --no-deps --document-private-items" $true
}

# 3. Packaging checks
if (-not $SkipPackaging) {
    Write-Header "Packaging Checks"

    Test-Command "Package creation succeeds" "cargo package --allow-dirty"
    Test-Command "Dry-run publish succeeds" "cargo publish --dry-run"
}

# 4. Metadata validation
Write-Header "Metadata Validation"

function Test-CargoField {
    param(
        [string]$Field,
        [string]$DisplayName
    )

    $script:TotalChecks++

    $content = Get-Content "Cargo.toml" -Raw
    if ($content -match "^$Field\s*=") {
        Write-Success "$DisplayName is present"
        $script:PassedChecks++
    } else {
        Write-Error "$DisplayName is missing"
        $script:FailedChecks++
    }
}

Test-CargoField "name" "Package name"
Test-CargoField "version" "Package version"
Test-CargoField "description" "Package description"
Test-CargoField "license" "Package license"
Test-CargoField "repository" "Repository URL"
Test-CargoField "documentation" "Documentation URL"
Test-CargoField "readme" "README file"
Test-CargoField "authors" "Package authors"
Test-CargoField "keywords" "Package keywords"
Test-CargoField "categories" "Package categories"

# 5. Code quality checks
Write-Header "Code Quality Checks"

Test-Command "Code formatting is correct" "cargo fmt --all -- --check"
Test-Command "Clippy lints pass" "cargo clippy --all-targets --all-features -- -D warnings"

# 6. Dependency checks
Write-Header "Dependency Checks"

# Check for git dependencies
$cargoContent = Get-Content "Cargo.toml" -Raw
if ($cargoContent -match 'git\s*=') {
    Write-Warning "Git dependencies found (not recommended for published crates)"
    $PassedChecks++
} else {
    Write-Success "No git dependencies"
    $PassedChecks++
}
$TotalChecks++

# 7. License and legal checks
Write-Header "License and Legal Checks"

# Check for LICENSE file
$licenseFiles = @("LICENSE", "LICENSE.md", "LICENSE.txt")
$licenseExists = $false
foreach ($licenseFile in $licenseFiles) {
    if (Test-Path $licenseFile) {
        $licenseExists = $true
        break
    }
}

if ($licenseExists) {
    Write-Success "License file exists"
    $PassedChecks++
} else {
    Write-Warning "License file not found (not required for crates.io but recommended)"
    $PassedChecks++
}
$TotalChecks++

# Check for README
if (Test-Path "README.md") {
    Write-Success "README.md exists"
    $PassedChecks++

    # Check README has basic content
    $readmeContent = Get-Content "README.md" -Raw
    if ($readmeContent -match '^#\s+') {
        Write-Success "README has title"
        $PassedChecks++
    } else {
        Write-Warning "README should have a proper title"
        $PassedChecks++
    }
    $TotalChecks += 2
} else {
    Write-Error "README.md is missing"
    $FailedChecks++
    $TotalChecks++
}

# 8. Security checks
Write-Header "Security Checks"

# Check for unsafe code
$unsafeFound = $false
if (Test-Path "src") {
    $unsafeFiles = Get-ChildItem -Path "src" -Recurse -Include "*.rs" |
                   Where-Object { (Get-Content $_.FullName -Raw) -match '\bunsafe\b' }
    if ($unsafeFiles) {
        Write-Warning "Unsafe code found in source files"
        $unsafeFound = $true
    }
}

if (-not $unsafeFound) {
    Write-Success "No unsafe code found"
}
$PassedChecks++
$TotalChecks++

# Check for TODO/FIXME
$todoFound = $false
if (Test-Path "src") {
    $todoFiles = Get-ChildItem -Path "src" -Recurse -Include "*.rs" |
                  Where-Object { (Get-Content $_.FullName -Raw) -match '\b(TODO|FIXME)\b' }
    if ($todoFiles) {
        Write-Warning "TODO/FIXME found in source files"
        $todoFound = $true
    }
}

if (-not $todoFound) {
    Write-Success "No TODO/FIXME in main code"
}
$PassedChecks++
$TotalChecks++

# 9. Performance checks
Write-Header "Performance Checks"

Test-Command "Release build succeeds" "cargo build --release"

# Final summary
Write-Header "Package Check Summary"

Write-Host ""
Write-Host "${Blue} Results:${Reset}"
Write-Host "   Total checks: $TotalChecks"
Write-Host "   Passed: $PassedChecks"
Write-Host "   Failed: $FailedChecks"

if ($FailedChecks -eq 0) {
    Write-Host ""
    Write-Success " All package sanity checks passed!"
    Write-Host ""
    Write-Info "Package is ready for publishing to crates.io"
    Write-Host ""
    Write-Host "${Blue} Next steps:${Reset}"
    Write-Host "   1. Update version in Cargo.toml if needed"
    Write-Host "   2. Update CHANGELOG.md with release notes"
    Write-Host "   3. Create a git tag: git tag v`$(grep '^version' Cargo.toml | cut -d'`"' -f2)"
    Write-Host "   4. Push tag to trigger release: git push origin --tags"
    Write-Host ""
    exit 0
} else {
    Write-Host ""
    Write-Error " $FailedChecks package check(s) failed!"
    Write-Host ""
    Write-Info "Please fix the issues above before publishing"
    Write-Host ""
    exit 1
}