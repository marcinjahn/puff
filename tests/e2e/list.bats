#!/usr/bin/env bats
load helpers

setup() { setup_puff_env; }
teardown() { teardown_puff_env; }

@test "list: succeeds with no projects" {
  run puff list
  assert_success
}

@test "list: shows associated project after init" {
  puff_init "myproject"
  run puff list
  assert_success
  assert_output_contains "myproject"
}

@test "list: shows multiple associated projects" {
  local project2
  project2="$(mktemp -d)"
  puff_init "project1"
  cd "$project2"
  puff_init "project2"
  cd "$PROJECT_DIR"
  run puff list
  assert_success
  assert_output_contains "project1"
  assert_output_contains "project2"
  rm -rf "$project2"
}

@test "list: shows unassociated project" {
  puff_init "myproject"
  mkdir -p "$PUFF_CONFIG_PATH/configs/orphan"
  run puff list
  assert_success
  assert_output_contains "orphan"
}

@test "list -a: shows only associated projects" {
  puff_init "myproject"
  mkdir -p "$PUFF_CONFIG_PATH/configs/orphan"
  run puff list -a
  assert_success
  assert_output_contains "myproject"
  assert_output_not_contains "orphan"
}

@test "list -u: shows only unassociated projects" {
  puff_init "myproject"
  mkdir -p "$PUFF_CONFIG_PATH/configs/orphan"
  run puff list -u
  assert_success
  assert_output_contains "orphan"
  assert_output_not_contains "myproject"
}

@test "list: project disappears after project forget" {
  puff_init "myproject"
  puff project forget -y myproject
  run puff list
  assert_success
  assert_output_not_contains "myproject"
}
