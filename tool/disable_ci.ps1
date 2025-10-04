$ErrorActionPreference = "SilentlyContinue"

Write-Host "Disabling CI/CD systems..."

# Remove GitHub Actions workflows if present
$gha = Join-Path $PSScriptRoot "..\.github\workflows"
if (Test-Path $gha) {
  Write-Host "Removing GitHub Actions workflows at $gha"
  Remove-Item -Recurse -Force $gha
}

# Remove the entire .github directory if it exists
$ghDir = Join-Path $PSScriptRoot "..\.github"
if (Test-Path $ghDir) {
  Write-Host "Removing .github directory at $ghDir"
  Remove-Item -Recurse -Force $ghDir
}

# Block re-creation by ignoring the folder
$gi = Join-Path $PSScriptRoot "..\.gitignore"
if (Test-Path $gi) {
  # Check if already ignored
  $content = Get-Content $gi -Raw
  if (-not $content.Contains(".github/")) {
    Add-Content $gi "`n# Disable CI`n.github/`n.github/workflows/`n"
    Write-Host "Added .github/ to .gitignore"
  }
} else {
  Set-Content $gi ".github/`n.github/workflows/`n"
  Write-Host "Created .gitignore with CI disabled"
}

# Remove common CI files (CircleCI, GitLab CI, etc.)
$ciCandidates = @(
  (Join-Path $PSScriptRoot "..\.circleci"),
  (Join-Path $PSScriptRoot "..\.gitlab-ci.yml"),
  (Join-Path $PSScriptRoot "..\azure-pipelines.yml"),
  (Join-Path $PSScriptRoot "..\Jenkinsfile"),
  (Join-Path $PSScriptRoot "..\.travis.yml"),
  (Join-Path $PSScriptRoot "..\appveyor.yml")
)

foreach ($c in $ciCandidates) {
  if (Test-Path $c) {
    Write-Host "Removing CI artifact: $c"
    Remove-Item -Recurse -Force $c
  }
}

Write-Host "CI disabled successfully."