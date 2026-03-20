#!/usr/bin/env bats
load helpers

setup() { setup_puff_env; }
teardown() { teardown_puff_env; }

@test "link: creates symlinks in another directory" {
  puff_init "myproject"
  echo "secret=123" >.env
  puff add .env

  local other_dir
  other_dir="$(mktemp -d)"
  cd "$other_dir"
  run puff link myproject
  assert_success
  assert_symlink "$other_dir/.env"
  assert_file_content "$other_dir/.env" "secret=123"

  rm -rf "$other_dir"
}

@test "link: file in subdirectory creates correct structure" {
  puff_init "myproject"
  mkdir -p config
  echo "db=postgres://localhost" >config/database.env
  puff add config/database.env

  local other_dir
  other_dir="$(mktemp -d)"
  cd "$other_dir"
  run puff link myproject
  assert_success
  assert_symlink "$other_dir/config/database.env"
  assert_file_content "$other_dir/config/database.env" "db=postgres://localhost"

  rm -rf "$other_dir"
}

@test "link: conflict creates backup and replaces with symlink" {
  puff_init "myproject"
  echo "managed=1" >.env
  puff add .env

  local other_dir
  other_dir="$(mktemp -d)"
  echo "local=2" >"$other_dir/.env"
  cd "$other_dir"
  run puff link myproject
  assert_success
  assert_symlink "$other_dir/.env"
  assert_file_content "$other_dir/.env" "managed=1"
  assert_file_exists "$other_dir/.env.bak"
  assert_file_content "$other_dir/.env.bak" "local=2"

  rm -rf "$other_dir"
}

@test "link: nonexistent project fails with error" {
  local other_dir
  other_dir="$(mktemp -d)"
  cd "$other_dir"
  run puff link no-such-project
  assert_failure

  rm -rf "$other_dir"
}

@test "link: from main directory fails with error" {
  puff_init "myproject"
  echo "secret=123" >.env
  puff add .env

  run puff link myproject
  assert_failure
  assert_output_contains "main directory"
}

@test "link: re-run picks up newly added files" {
  puff_init "myproject"
  echo "first=1" >.env
  puff add .env

  local other_dir
  other_dir="$(mktemp -d)"
  cd "$other_dir"
  puff link myproject
  assert_symlink "$other_dir/.env"

  cd "$PROJECT_DIR"
  echo "second=2" >.secrets
  puff add .secrets

  cd "$other_dir"
  run puff link myproject
  assert_success
  assert_symlink "$other_dir/.env"
  assert_symlink "$other_dir/.secrets"
  assert_not_exists "$other_dir/.env.bak"

  rm -rf "$other_dir"
}

@test "link: project with no managed files prints message" {
  puff_init "myproject"

  local other_dir
  other_dir="$(mktemp -d)"
  cd "$other_dir"
  run puff link myproject
  assert_success
  assert_output_contains "no managed files"

  rm -rf "$other_dir"
}

@test "link: multiple files are all linked" {
  puff_init "myproject"
  echo "a=1" >.env
  echo "b=2" >.secrets
  puff add .env .secrets

  local other_dir
  other_dir="$(mktemp -d)"
  cd "$other_dir"
  run puff link myproject
  assert_success
  assert_symlink "$other_dir/.env"
  assert_symlink "$other_dir/.secrets"
  assert_file_content "$other_dir/.env" "a=1"
  assert_file_content "$other_dir/.secrets" "b=2"

  rm -rf "$other_dir"
}
