#!/usr/bin/env bash
set -euo pipefail

root="$(dirname "$(readlink -f "$0")")"
repo="$(readlink -f "$root/..")"
gha="$repo/.github"

echo "=== Ensuring No CI/Actions (Phase 4 Policy) ==="

if [ -d "$gha" ]; then
    rm -rf "$gha"
    echo "✅ Removed .github (GitHub Actions) directory."
else
    echo "✅ No .github directory found."
fi

# Check for other CI directories and files
ci_paths=(
    ".circleci"
    ".gitlab-ci.yml"
    "azure-pipelines.yml"
    "Jenkinsfile"
    ".githubworkflows"
)

for ci_path in "${ci_paths[@]}"; do
    full_path="$repo/$ci_path"
    if [ -e "$full_path" ]; then
        if [ -d "$full_path" ]; then
            rm -rf "$full_path"
            echo "✅ Removed CI directory: $ci_path"
        else
            rm -f "$full_path"
            echo "✅ Removed CI file: $ci_path"
        fi
    fi
done

gi="$repo/.gitignore"
block="
# Block CI/Actions (Phase 4 policy)
.github/
.github/workflows/
.circleci/
.gitlab-ci.yml
azure-pipelines.yml
Jenkinsfile"

if [ -f "$gi" ]; then
    echo "$block" >> "$gi"
    echo "✅ Added CI blocking rules to .gitignore"
else
    echo "$block" > "$gi"
    echo "✅ Created .gitignore with CI blocking rules"
fi

echo "✅ CI/Actions disabled & ignored."