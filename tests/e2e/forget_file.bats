#!/usr/bin/env bats
load helpers

setup() { setup_puff_env; }
teardown() { teardown_puff_env; }

@test "forget file: restores file content to project dir" {
  puff_init "myproject"
  echo "secret=123" >.env
  puff add .env
  run puff forget .env
  assert_success
  assert_file_content "$PROJECT_DIR/.env" "secret=123"
}

@test "forget file: replaces symlink with regular file" {
  puff_init "myproject"
  echo "secret=123" >.env
  puff add .env
  puff forget .env
  assert_not_symlink "$PROJECT_DIR/.env"
  assert_file_exists "$PROJECT_DIR/.env"
}

@test "forget file: removes file from managed dir" {
  puff_init "myproject"
  echo "secret=123" >.env
  puff add .env
  puff forget .env
  assert_not_exists "$PUFF_CONFIG_PATH/configs/myproject/.env"
}

@test "forget file: with --delete removes file without restoring" {
  puff_init "myproject"
  echo "secret=123" >.env
  puff add .env
  run puff forget --delete .env
  assert_success
  assert_not_exists "$PROJECT_DIR/.env"
}

@test "forget file: fails for file not managed by puff" {
  puff_init "myproject"
  echo "test" >somefile.txt
  run puff forget somefile.txt
  assert_failure
}

@test "forget file: cleans up empty subdir in managed dir" {
  puff_init "myproject"
  mkdir -p config
  echo "val=1" >config/settings.env
  puff add config/settings.env
  puff forget config/settings.env
  assert_not_exists "$PUFF_CONFIG_PATH/configs/myproject/config"
}
