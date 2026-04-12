#!/usr/bin/env bash
set -euo pipefail

# Updates CHANGELOG.md for a release: moves [Unreleased] content under a new
# version heading and updates comparison links.
#
# Usage: update-changelog.sh <version> <repo-url> [changelog-path]
#   changelog-path defaults to CHANGELOG.md

version="${1:?Usage: update-changelog.sh <version> <repo-url> [changelog-path]}"
repo="${2:?Usage: update-changelog.sh <version> <repo-url> [changelog-path]}"
changelog="${3:-CHANGELOG.md}"
date=$(date +%Y-%m-%d)

# Validate that [Unreleased] has content
content=$(sed -n '/^## \[Unreleased\]/,/^## \[/{/^## \[/!p}' "$changelog" | grep -v '^$' || true)
if [ -z "$content" ]; then
  echo "Error: no content in [Unreleased] section of $changelog" >&2
  exit 1
fi

# Insert new version heading after [Unreleased]
sed -i "s/^## \[Unreleased\]/## [Unreleased]\n\n## [$version] - $date/" "$changelog"

# Extract previous version from the unreleased comparison link
prev_version=$(grep '^\[unreleased\]:' "$changelog" | grep -oP 'compare/v\K[0-9]+\.[0-9]+\.[0-9]+(?=\.\.\.)')

# Update unreleased link to compare from new version
sed -i "s|\[unreleased\]:.*|[unreleased]: $repo/compare/v$version...HEAD|" "$changelog"

# Insert new version comparison link
sed -i "/^\[unreleased\]:/a [$version]: $repo/compare/v$prev_version...v$version" "$changelog"
