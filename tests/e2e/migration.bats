#!/usr/bin/env bats
load helpers

setup() { setup_puff_env; }
teardown() { teardown_puff_env; }

@test "migration: old configs/ is moved to projects/" {
  # Manually set up old-style layout
  mkdir -p "$PUFF_CONFIG_PATH/configs/myproject"
  echo "SECRET=1" >"$PUFF_CONFIG_PATH/configs/myproject/.env"
  echo '{"projects":[]}' >"$PUFF_CONFIG_PATH/config.json"

  run puff list
  assert_success

  # Old dir should be gone, new dir should have the file
  assert_not_exists "$PUFF_CONFIG_PATH/configs"
  assert_file_exists "$PUFF_DATA_PATH/projects/myproject/.env"
  assert_file_content "$PUFF_DATA_PATH/projects/myproject/.env" "SECRET=1"
}

@test "migration: migration message printed" {
  mkdir -p "$PUFF_CONFIG_PATH/configs/myproject"
  echo '{"projects":[]}' >"$PUFF_CONFIG_PATH/config.json"

  run puff list
  assert_success
  assert_output_contains "Managed projects have been migrated"
}

@test "migration: no migration when no legacy dir" {
  run puff list
  assert_success
  assert_output_not_contains "migrated"
}

@test "migration: fails when both dirs exist" {
  mkdir -p "$PUFF_CONFIG_PATH/configs/myproject"
  mkdir -p "$PUFF_DATA_PATH/projects/myproject"
  echo '{"projects":[]}' >"$PUFF_CONFIG_PATH/config.json"

  run puff list
  assert_failure
  assert_output_contains "Both legacy"
}

@test "migration: symlinks updated after migration" {
  # Set up old-style layout with an associated project + symlink.
  # Use native_path so that config.json and symlink targets contain
  # Windows-native paths when running under MSYS/Git Bash — the Rust
  # binary cannot resolve MSYS-style paths like /tmp/tmp.XXX.
  mkdir -p "$PUFF_CONFIG_PATH/configs/myproject"
  echo "SECRET=1" >"$PUFF_CONFIG_PATH/configs/myproject/.env"
  cat >"$PUFF_CONFIG_PATH/config.json" <<EOF
{"projects":[{"name":"myproject","id":"1","path":"$(native_path "$PROJECT_DIR")"}]}
EOF

  # On Windows (MSYS/Git Bash), ln -s creates file copies by default (deepcopy
  # mode), not real NTFS symlinks. The Rust binary needs real symlinks.
  export MSYS="${MSYS:+$MSYS }winsymlinks:nativestrict"

  # Create symlink in project dir pointing to old location
  ln -s "$(native_path "$PUFF_CONFIG_PATH/configs/myproject/.env")" "$PROJECT_DIR/.env"

  run puff list
  assert_success

  # Symlink should still work and now point to new location
  assert_symlink "$PROJECT_DIR/.env"
  assert_file_content "$PROJECT_DIR/.env" "SECRET=1"
  local target
  target="$(native_path "$(readlink "$PROJECT_DIR/.env")")"
  echo "$target" | grep -qF "$(native_path "$PUFF_DATA_PATH")/projects/"
}
