#!/usr/bin/env bats
load helpers

setup() { setup_puff_env; }
teardown() { teardown_puff_env; }

@test "init: creates config.json with project entry" {
  run bash -c "echo 'myproject' | puff init"
  assert_success
  assert_file_exists "$PUFF_CONFIG_PATH/config.json"
  run grep -q '"myproject"' "$PUFF_CONFIG_PATH/config.json"
  assert_success
}

@test "init: creates managed directory" {
  puff_init "myproject"
  assert_file_exists "$PUFF_CONFIG_PATH/configs/myproject"
}

@test "init: project appears in puff list" {
  puff_init "myproject"
  run puff list
  assert_success
  assert_output_contains "myproject"
}

@test "init: fails when directory is already initialized" {
  puff_init "myproject"
  run bash -c "echo 'myproject' | puff init"
  assert_failure
}

@test "init: accepts directory name as default when empty input given" {
  local named_dir
  named_dir="$(mktemp -d)"
  # Send empty line — puff should fall back to using the directory's basename
  run bash -c "echo '' | PUFF_CONFIG_PATH='$PUFF_CONFIG_PATH' puff init"
  assert_success
  rm -rf "$named_dir"
}

@test "init: new machine - associates with unassociated project and recreates symlinks" {
  # Set up: project with a managed file
  puff_init "myproject"
  echo "secret=abc" >.env
  puff add .env

  # Simulate new machine: copy puff home but wipe config associations
  local new_puff_home
  new_puff_home="$(mktemp -d)"
  cp -r "$PUFF_CONFIG_PATH/." "$new_puff_home/"
  echo '{"projects":[]}' >"$new_puff_home/config.json"

  # Remove symlink (simulates fresh checkout, no local symlink yet)
  rm "$PROJECT_DIR/.env"

  # Init: should detect unassociated project and ask to select; send "1" to pick it
  run bash -c "echo '1' | PUFF_CONFIG_PATH='$new_puff_home' puff init"
  assert_success
  assert_symlink "$PROJECT_DIR/.env"
  assert_file_content "$PROJECT_DIR/.env" "secret=abc"

  rm -rf "$new_puff_home"
}
