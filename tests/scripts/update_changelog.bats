#!/usr/bin/env bats

load helpers

setup() {
  tmp_changelog="$(mktemp)"
  cat >"$tmp_changelog" <<'EOF'
# Changelog

## [Unreleased]

### Added

- New feature A
- New feature B

### Fixed

- Bug fix C

## [1.0.0] - 2025-01-01

### Added

- Initial release

[unreleased]: https://github.com/example/repo/compare/v1.0.0...HEAD
[1.0.0]: https://github.com/example/repo/releases/tag/v1.0.0
EOF
}

teardown() {
  rm -f "$tmp_changelog"
}

@test "inserts new version heading after Unreleased" {
  run "$SCRIPTS_DIR/update-changelog.sh" "1.1.0" "https://github.com/example/repo" "$tmp_changelog"
  assert_success
  grep -q '## \[Unreleased\]' "$tmp_changelog"
  grep -q '## \[1.1.0\]' "$tmp_changelog"
}

@test "new version heading includes today's date" {
  run "$SCRIPTS_DIR/update-changelog.sh" "1.1.0" "https://github.com/example/repo" "$tmp_changelog"
  assert_success
  today=$(date +%Y-%m-%d)
  grep -q "## \[1.1.0\] - $today" "$tmp_changelog"
}

@test "unreleased section is empty after update" {
  run "$SCRIPTS_DIR/update-changelog.sh" "1.1.0" "https://github.com/example/repo" "$tmp_changelog"
  assert_success
  # Content between [Unreleased] and [1.1.0] should be empty
  content=$(sed -n '/^## \[Unreleased\]/,/^## \[1\.1\.0\]/{/^## \[/!p}' "$tmp_changelog" | grep -v '^$' || true)
  if [ -n "$content" ]; then
    echo "Expected empty Unreleased section, got: $content" >&2
    return 1
  fi
}

@test "preserves existing version content" {
  run "$SCRIPTS_DIR/update-changelog.sh" "1.1.0" "https://github.com/example/repo" "$tmp_changelog"
  assert_success
  grep -q '## \[1.0.0\] - 2025-01-01' "$tmp_changelog"
  grep -q 'Initial release' "$tmp_changelog"
}

@test "updates unreleased comparison link" {
  run "$SCRIPTS_DIR/update-changelog.sh" "1.1.0" "https://github.com/example/repo" "$tmp_changelog"
  assert_success
  grep -q '\[unreleased\]: https://github.com/example/repo/compare/v1.1.0\.\.\.HEAD' "$tmp_changelog"
}

@test "adds new version comparison link" {
  run "$SCRIPTS_DIR/update-changelog.sh" "1.1.0" "https://github.com/example/repo" "$tmp_changelog"
  assert_success
  grep -q '\[1.1.0\]: https://github.com/example/repo/compare/v1.0.0\.\.\.v1.1.0' "$tmp_changelog"
}

@test "preserves existing comparison links" {
  run "$SCRIPTS_DIR/update-changelog.sh" "1.1.0" "https://github.com/example/repo" "$tmp_changelog"
  assert_success
  grep -q '\[1.0.0\]: https://github.com/example/repo/releases/tag/v1.0.0' "$tmp_changelog"
}

@test "fails if unreleased section is empty" {
  cat >"$tmp_changelog" <<'EOF'
# Changelog

## [Unreleased]

## [1.0.0] - 2025-01-01

[unreleased]: https://github.com/example/repo/compare/v1.0.0...HEAD
EOF
  run "$SCRIPTS_DIR/update-changelog.sh" "1.1.0" "https://github.com/example/repo" "$tmp_changelog"
  assert_failure
  assert_output_contains "no content in [Unreleased]"
}

@test "released content appears under new version heading" {
  run "$SCRIPTS_DIR/update-changelog.sh" "1.1.0" "https://github.com/example/repo" "$tmp_changelog"
  assert_success
  today=$(date +%Y-%m-%d)
  # Content between [1.1.0] and [1.0.0] should contain the features
  content=$(sed -n "/^## \[1.1.0\] - $today/,/^## \[1.0.0\]/{/^## \[/!p}" "$tmp_changelog")
  echo "$content" | grep -q 'New feature A'
  echo "$content" | grep -q 'Bug fix C'
}
