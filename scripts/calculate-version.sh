#!/usr/bin/env bash
set -euo pipefail

# Calculates release and next-dev versions from a Cargo.toml -dev version.
#
# Usage: calculate-version.sh <bump> <cargo-toml-path>
#   bump: major | minor | patch
#
# Outputs (one per line):
#   release_version
#   next_dev_version

bump="${1:?Usage: calculate-version.sh <major|minor|patch> <cargo-toml-path>}"
cargo_toml="${2:?Usage: calculate-version.sh <major|minor|patch> <cargo-toml-path>}"

current=$(sed -n '/^\[package\]/,/^\[/{s/^version = "\(.*\)"/\1/p}' "$cargo_toml")

if [ -z "$current" ]; then
  echo "Error: could not read version from $cargo_toml" >&2
  exit 1
fi

base="${current%-dev}"
if [ "$base" = "$current" ]; then
  echo "Error: version ($current) does not end with -dev" >&2
  exit 1
fi

IFS='.' read -r major minor patch <<<"$base"

case "$bump" in
major)
  major=$((major + 1))
  minor=0
  patch=0
  ;;
minor)
  minor=$((minor + 1))
  patch=0
  ;;
patch)
  ;;
*)
  echo "Error: invalid bump type '$bump' (expected major, minor, or patch)" >&2
  exit 1
  ;;
esac

version="${major}.${minor}.${patch}"
next_dev="${major}.${minor}.$((patch + 1))-dev"

echo "$version"
echo "$next_dev"
