#!/usr/bin/env bats
load helpers

setup() { setup_puff_env; }
teardown() { teardown_puff_env; }

@test "project forget: restores all managed files" {
  puff_init "myproject"
  echo "secret=123" >.env
  echo "config=value" >app.conf
  puff add .env
  puff add app.conf
  run puff project forget -y myproject
  assert_success
  assert_file_content "$PROJECT_DIR/.env" "secret=123"
  assert_file_content "$PROJECT_DIR/app.conf" "config=value"
}

@test "project forget: replaces symlinks with regular files" {
  puff_init "myproject"
  echo "secret=123" >.env
  puff add .env
  puff project forget -y myproject
  assert_not_symlink "$PROJECT_DIR/.env"
  assert_file_exists "$PROJECT_DIR/.env"
}

@test "project forget: removes managed directory" {
  puff_init "myproject"
  echo "secret=123" >.env
  puff add .env
  puff project forget -y myproject
  assert_not_exists "$PUFF_CONFIG_PATH/configs/myproject"
}

@test "project forget: removes project from config" {
  puff_init "myproject"
  puff project forget -y myproject
  run puff list
  assert_success
  assert_output_not_contains "myproject"
}

@test "project forget: --delete-files removes files without restoring" {
  puff_init "myproject"
  echo "secret=123" >.env
  puff add .env
  run puff project forget --delete-files -y myproject
  assert_success
  assert_not_exists "$PROJECT_DIR/.env"
}

@test "project forget: non-existent project fails with a clear message" {
  run puff project forget -y nonexistent
  assert_failure
  assert_output_contains "nonexistent"
}
