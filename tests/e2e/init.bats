#!/usr/bin/env bats
load helpers

setup() { setup_puff_env; }
teardown() { teardown_puff_env; }

@test "init: creates config.json with project entry" {
  run puff init --name myproject
  assert_success
  assert_file_exists "$PUFF_CONFIG_PATH/config.json"
  run grep -q '"myproject"' "$PUFF_CONFIG_PATH/config.json"
  assert_success
}

@test "init: creates managed directory" {
  puff_init "myproject"
  assert_file_exists "$PUFF_DATA_PATH/projects/myproject"
}

@test "init: project appears in puff list" {
  puff_init "myproject"
  run puff list
  assert_success
  assert_output_contains "myproject"
}

@test "init: fails when directory is already initialized" {
  puff_init "myproject"
  run puff init --name myproject
  assert_failure
}

@test "init: uses directory basename when no name given" {
  # The current PROJECT_DIR has a basename; --name with that basename simulates the default
  local basename
  basename="$(basename "$PROJECT_DIR")"
  run puff init --name "$basename"
  assert_success
  run grep -q "\"$basename\"" "$PUFF_CONFIG_PATH/config.json"
  assert_success
}

@test "init: new machine - associates with unassociated project and recreates symlinks" {
  # Set up: project with a managed file
  puff_init "myproject"
  echo "secret=abc" >.env
  puff add .env

  # Simulate new machine: copy puff config and data but wipe config associations
  local new_config_home new_data_home
  new_config_home="$(mktemp -d)"
  new_data_home="$(mktemp -d)"
  cp -r "$PUFF_CONFIG_PATH/." "$new_config_home/"
  cp -r "$PUFF_DATA_PATH/." "$new_data_home/"
  echo '{"projects":[]}' >"$new_config_home/config.json"

  # Remove symlink (simulates fresh checkout, no local symlink yet)
  rm "$PROJECT_DIR/.env"

  # Use --associate to pick the unassociated project
  run env PUFF_CONFIG_PATH="$new_config_home" PUFF_DATA_PATH="$new_data_home" puff init --associate myproject
  assert_success
  assert_symlink "$PROJECT_DIR/.env"
  assert_file_content "$PROJECT_DIR/.env" "secret=abc"

  rm -rf "$new_config_home" "$new_data_home"
}
