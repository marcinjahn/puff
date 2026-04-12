#!/usr/bin/env bats

load helpers

setup() {
  tmp_toml="$(mktemp)"
  cat >"$tmp_toml" <<'EOF'
[package]
name = "test"
version = "1.2.3-dev"
edition = "2024"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
EOF
}

teardown() {
  rm -f "$tmp_toml"
}

@test "patch bump keeps base version" {
  run "$SCRIPTS_DIR/calculate-version.sh" patch "$tmp_toml"
  assert_success
  assert_line 0 "1.2.3"
  assert_line 1 "1.2.4-dev"
}

@test "minor bump increments minor and resets patch" {
  run "$SCRIPTS_DIR/calculate-version.sh" minor "$tmp_toml"
  assert_success
  assert_line 0 "1.3.0"
  assert_line 1 "1.3.1-dev"
}

@test "major bump increments major and resets minor and patch" {
  run "$SCRIPTS_DIR/calculate-version.sh" major "$tmp_toml"
  assert_success
  assert_line 0 "2.0.0"
  assert_line 1 "2.0.1-dev"
}

@test "fails if version does not end with -dev" {
  cat >"$tmp_toml" <<'EOF'
[package]
name = "test"
version = "1.2.3"
EOF
  run "$SCRIPTS_DIR/calculate-version.sh" patch "$tmp_toml"
  assert_failure
  assert_output_contains "does not end with -dev"
}

@test "fails with invalid bump type" {
  run "$SCRIPTS_DIR/calculate-version.sh" invalid "$tmp_toml"
  assert_failure
  assert_output_contains "invalid bump type"
}

@test "fails if cargo toml has no version" {
  cat >"$tmp_toml" <<'EOF'
[package]
name = "test"
EOF
  run "$SCRIPTS_DIR/calculate-version.sh" patch "$tmp_toml"
  assert_failure
  assert_output_contains "could not read version"
}

@test "reads version from package section not dependencies" {
  cat >"$tmp_toml" <<'EOF'
[dependencies]
version = "9.9.9"

[package]
name = "test"
version = "0.5.0-dev"
EOF
  run "$SCRIPTS_DIR/calculate-version.sh" patch "$tmp_toml"
  assert_success
  assert_line 0 "0.5.0"
}

@test "major bump from 0.x" {
  cat >"$tmp_toml" <<'EOF'
[package]
version = "0.2.0-dev"
EOF
  run "$SCRIPTS_DIR/calculate-version.sh" major "$tmp_toml"
  assert_success
  assert_line 0 "1.0.0"
  assert_line 1 "1.0.1-dev"
}
