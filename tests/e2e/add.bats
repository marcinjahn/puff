#!/usr/bin/env bats
load helpers

setup() { setup_puff_env; }
teardown() { teardown_puff_env; }

@test "add: non-existent file creates empty managed file and symlink" {
  puff_init "myproject"
  run puff add .env
  assert_success
  assert_symlink "$PROJECT_DIR/.env"
  assert_file_exists "$PUFF_DATA_PATH/projects/myproject/.env"
}

@test "add: existing file copies content to managed dir" {
  puff_init "myproject"
  echo "secret=123" >.env
  run puff add .env
  assert_success
  assert_file_content "$PUFF_DATA_PATH/projects/myproject/.env" "secret=123"
}

@test "add: existing file is replaced with symlink" {
  puff_init "myproject"
  echo "secret=123" >.env
  puff add .env
  assert_symlink "$PROJECT_DIR/.env"
}

@test "add: symlink resolves to correct content" {
  puff_init "myproject"
  echo "secret=123" >.env
  puff add .env
  assert_file_content "$PROJECT_DIR/.env" "secret=123"
}

@test "add: --git-ignore creates .gitignore with file entry" {
  puff_init "myproject"
  run puff add --git-ignore .env
  assert_success
  assert_file_exists ".gitignore"
  run grep -q '\.env' .gitignore
  assert_success
}

@test "add: --git-ignore appends to existing .gitignore" {
  puff_init "myproject"
  echo "node_modules" >.gitignore
  puff add --git-ignore .env
  run grep -q 'node_modules' .gitignore
  assert_success
  run grep -q '\.env' .gitignore
  assert_success
}

@test "add: file in subdirectory creates correct symlink and managed file" {
  puff_init "myproject"
  mkdir -p config
  echo "db_url=postgres://localhost" >config/database.env
  run puff add config/database.env
  assert_success
  assert_symlink "$PROJECT_DIR/config/database.env"
  assert_file_exists "$PUFF_DATA_PATH/projects/myproject/config/database.env"
}

@test "add: file already in managed dir creates symlink without error" {
  puff_init "myproject"
  # Simulate managed file already existing (e.g. copied from another machine)
  echo "token=xyz" >"$PUFF_DATA_PATH/projects/myproject/.env"
  run puff add .env
  assert_success
  assert_symlink "$PROJECT_DIR/.env"
  assert_file_content "$PROJECT_DIR/.env" "token=xyz"
}

@test "add: fails in non-initialized directory" {
  run puff add .env
  assert_failure
}

@test "add: multiple files are all added" {
  puff_init "myproject"
  echo "a=1" >.env
  echo "b=2" >.env.local
  run puff add .env .env.local
  assert_success
  assert_symlink "$PROJECT_DIR/.env"
  assert_symlink "$PROJECT_DIR/.env.local"
  assert_file_exists "$PUFF_DATA_PATH/projects/myproject/.env"
  assert_file_exists "$PUFF_DATA_PATH/projects/myproject/.env.local"
}

@test "add: conflict when file exists in both places does not print success" {
  puff_init "myproject"
  echo "local=1" >.env
  # Simulate managed file already existing with different content
  echo "managed=2" >"$PUFF_DATA_PATH/projects/myproject/.env"
  run puff add .env
  assert_failure
  assert_output_contains "Conflict"
  assert_output_not_contains "Added"
  # .gitignore should NOT be created on conflict
  assert_not_exists ".gitignore"
}

@test "add: already managed file does not print Added" {
  puff_init "myproject"
  echo "secret=123" >.env
  puff add .env
  run puff add --git-ignore .env
  assert_success
  assert_output_contains "already managed"
  assert_output_not_contains "Added"
  # .gitignore should NOT be touched for already-managed file
  assert_not_exists ".gitignore"
}

@test "add: partial failure exits with code 1 and processes remaining files" {
  puff_init "myproject"
  echo "a=1" >.env
  echo "b=2" >.secrets
  run puff add .env nonexistent-dir/ .secrets
  assert_failure
  assert_symlink "$PROJECT_DIR/.env"
  assert_symlink "$PROJECT_DIR/.secrets"
  assert_output_contains "Error:"
}
