$ErrorActionPreference = "SilentlyContinue"
$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$repo = Resolve-Path (Join-Path $root "..")
$gha = Join-Path $repo ".github"

Write-Host "=== Ensuring No CI/Actions (Phase 4 Policy) ==="

if (Test-Path $gha) {
  Remove-Item -Recurse -Force $gha
  Write-Host "Removed .github (GitHub Actions) directory."
} else {
  Write-Host "No .github directory found."
}

# Check for other CI directories and files
$ciPaths = @(
    ".circleci",
    ".gitlab-ci.yml",
    "azure-pipelines.yml",
    "Jenkinsfile",
    ".githubworkflows"
)

foreach ($ciPath in $ciPaths) {
    $fullPath = Join-Path $repo $ciPath
    if (Test-Path $fullPath) {
        if (Test-Path $fullPath -PathType Container) {
            Remove-Item -Recurse -Force $fullPath
            Write-Host "Removed CI directory: $ciPath"
        } else {
            Remove-Item -Force $fullPath
            Write-Host "Removed CI file: $ciPath"
        }
    }
}

$gi = Join-Path $repo ".gitignore"
$block = @"

# Block CI/Actions (Phase 4 policy)
.github/
.github/workflows/
.circleci/
.gitlab-ci.yml
azure-pipelines.yml
Jenkinsfile
"@

if (Test-Path $gi) {
    Add-Content $gi $block
    Write-Host "Added CI blocking rules to .gitignore"
} else {
    Set-Content $gi $block
    Write-Host "Created .gitignore with CI blocking rules"
}

Write-Host "CI/Actions disabled & ignored."