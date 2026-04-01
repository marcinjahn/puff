#!/usr/bin/env bats
load helpers

setup() { setup_puff_env; }
teardown() { teardown_puff_env; }

# --- add directory ---

@test "add dir: existing directory is replaced with symlink" {
  puff_init "myproject"
  mkdir -p config
  echo "DB_URL=postgres" >config/db.env
  echo "APP_KEY=secret" >config/app.env

  run puff add config
  assert_success
  assert_symlink "$PROJECT_DIR/config"
  assert_file_content "$PROJECT_DIR/config/db.env" "DB_URL=postgres"
  assert_file_content "$PROJECT_DIR/config/app.env" "APP_KEY=secret"
}

@test "add dir: contents are preserved in data store" {
  puff_init "myproject"
  mkdir -p config
  echo "DB_URL=postgres" >config/db.env
  puff add config

  assert_file_content "$PUFF_DATA_PATH/projects/myproject/config/db.env" "DB_URL=postgres"
}

@test "add dir: fresh directory with --dir creates empty dir and symlink" {
  puff_init "myproject"
  run puff add --dir staging
  assert_success
  assert_symlink "$PROJECT_DIR/staging"
  assert_file_exists "$PUFF_DATA_PATH/projects/myproject/staging"
}

@test "add dir: --dir on existing file produces error" {
  puff_init "myproject"
  echo "content" >somefile
  run puff add --dir somefile
  assert_failure
  assert_output_contains "exists and is a file"
}

@test "add dir: --git-ignore adds directory with trailing slash" {
  puff_init "myproject"
  mkdir -p config
  echo "val=1" >config/settings.env
  run puff add --git-ignore config
  assert_success
  assert_file_exists ".gitignore"
  run grep -q 'config/' .gitignore
  assert_success
}

@test "add dir: auto-absorbs individually managed files" {
  puff_init "myproject"
  mkdir -p config
  echo "DB_URL=postgres" >config/db.env
  echo "APP_KEY=secret" >config/app.env
  puff add config/db.env
  assert_symlink "$PROJECT_DIR/config/db.env"

  run puff add config
  assert_success
  assert_symlink "$PROJECT_DIR/config"
  assert_file_content "$PROJECT_DIR/config/db.env" "DB_URL=postgres"
  assert_file_content "$PROJECT_DIR/config/app.env" "APP_KEY=secret"
}

@test "add dir: adding file inside managed directory fails" {
  puff_init "myproject"
  mkdir -p config
  echo "val=1" >config/db.env
  puff add config

  run puff add config/db.env
  assert_failure
  assert_output_contains "already managed as part of directory"
}

@test "add dir: nested subdirectories are preserved" {
  puff_init "myproject"
  mkdir -p config/sub/deep
  echo "nested=true" >config/sub/deep/secret.env
  puff add config

  assert_symlink "$PROJECT_DIR/config"
  assert_file_content "$PROJECT_DIR/config/sub/deep/secret.env" "nested=true"
}

# --- forget directory ---

@test "forget dir: restores real directory from managed dir" {
  puff_init "myproject"
  mkdir -p config
  echo "val=1" >config/db.env
  puff add config

  run puff forget config
  assert_success
  assert_not_symlink "$PROJECT_DIR/config"
  assert_file_exists "$PROJECT_DIR/config"
  assert_file_content "$PROJECT_DIR/config/db.env" "val=1"
}

@test "forget dir: removes managed directory from data store" {
  puff_init "myproject"
  mkdir -p config
  echo "val=1" >config/db.env
  puff add config
  puff forget config

  assert_not_exists "$PUFF_DATA_PATH/projects/myproject/config"
}

@test "forget dir: with --delete removes without restoring" {
  puff_init "myproject"
  mkdir -p config
  echo "val=1" >config/db.env
  puff add config

  run puff forget --delete config
  assert_success
  assert_not_exists "$PROJECT_DIR/config"
  assert_not_exists "$PUFF_DATA_PATH/projects/myproject/config"
}

@test "forget dir: forgetting file inside managed directory fails" {
  puff_init "myproject"
  mkdir -p config
  echo "val=1" >config/db.env
  puff add config

  run puff forget config/db.env
  assert_failure
  assert_output_contains "part of managed directory"
}

# --- status ---

@test "add dir: status shows directory with trailing slash" {
  puff_init "myproject"
  mkdir -p config
  echo "val=1" >config/db.env
  puff add config

  run puff status
  assert_success
  assert_output_contains "config/"
  assert_output_contains "Managed items:"
}

# --- init/link with managed directories ---

@test "add dir: init recreates directory symlink for managed dirs" {
  puff_init "myproject"
  mkdir -p config
  echo "val=1" >config/db.env
  puff add config
  echo "regular=1" >.env
  puff add .env

  # Simulate new machine: remove symlinks, re-associate
  rm "$PROJECT_DIR/config"
  rm "$PROJECT_DIR/.env"
  puff project forget -y --delete-files myproject

  # Re-create managed data
  local managed="$PUFF_DATA_PATH/projects/myproject"
  mkdir -p "$managed/config"
  echo "val=1" >"$managed/config/db.env"
  printf 'config\n' >"$managed/.puff_managed_dirs"
  echo "regular=1" >"$managed/.env"

  run puff init --associate myproject
  assert_success
  assert_symlink "$PROJECT_DIR/config"
  assert_symlink "$PROJECT_DIR/.env"
  assert_file_content "$PROJECT_DIR/config/db.env" "val=1"
}

@test "add dir: link creates directory symlink in another directory" {
  puff_init "myproject"
  mkdir -p config
  echo "val=1" >config/db.env
  puff add config

  local other_dir
  other_dir="$(mktemp -d)"
  cd "$other_dir"
  run puff link myproject
  assert_success
  assert_symlink "$other_dir/config"
  assert_file_content "$other_dir/config/db.env" "val=1"
  assert_output_contains "item"

  rm -rf "$other_dir"
}

@test "add dir: link with conflicting real directory creates backup" {
  puff_init "myproject"
  mkdir -p config
  echo "managed=1" >config/db.env
  puff add config

  local other_dir
  other_dir="$(mktemp -d)"
  mkdir -p "$other_dir/config"
  echo "local=2" >"$other_dir/config/local.env"
  cd "$other_dir"
  run puff link myproject
  assert_success
  assert_symlink "$other_dir/config"
  assert_file_content "$other_dir/config/db.env" "managed=1"
  assert_file_exists "$other_dir/config.bak"
  assert_file_content "$other_dir/config.bak/local.env" "local=2"

  rm -rf "$other_dir"
}

# --- project forget with managed directories ---

@test "add dir: project forget restores managed directories" {
  puff_init "myproject"
  mkdir -p config
  echo "val=1" >config/db.env
  echo "regular=1" >.env
  puff add config
  puff add .env

  run puff project forget -y myproject
  assert_success
  assert_not_symlink "$PROJECT_DIR/config"
  assert_file_exists "$PROJECT_DIR/config/db.env"
  assert_file_content "$PROJECT_DIR/config/db.env" "val=1"
  assert_not_symlink "$PROJECT_DIR/.env"
  assert_file_exists "$PROJECT_DIR/.env"
}

@test "add dir: project forget --delete-files removes directory symlinks" {
  puff_init "myproject"
  mkdir -p config
  echo "val=1" >config/db.env
  puff add config

  run puff project forget -y --delete-files myproject
  assert_success
  assert_not_exists "$PROJECT_DIR/config"
}
