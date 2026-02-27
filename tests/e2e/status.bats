#!/usr/bin/env bats
load helpers

setup() { setup_puff_env; }
teardown() { teardown_puff_env; }

@test "status: reports not managed when no project initialized" {
  run puff status
  assert_success
  assert_output_contains "not managed"
}

@test "status: reports project name after init" {
  puff_init "myproject"
  run puff status
  assert_success
  assert_output_contains "myproject"
}

@test "status: reports no files when project has none" {
  puff_init "myproject"
  run puff status
  assert_success
  assert_output_contains "(none)"
}

@test "status: lists managed files" {
  puff_init "myproject"
  puff add .env
  run puff status
  assert_success
  assert_output_contains ".env"
}

@test "status: lists multiple managed files" {
  puff_init "myproject"
  echo "a=1" >.env
  echo "b=2" >.env.local
  puff add .env .env.local
  run puff status
  assert_success
  assert_output_contains ".env"
  assert_output_contains ".env.local"
}

@test "status: works from subdirectory of project" {
  puff_init "myproject"
  mkdir -p subdir
  cd subdir
  run puff status
  assert_success
  assert_output_contains "myproject"
}
