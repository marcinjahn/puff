REPO_ROOT="$(cd "$BATS_TEST_DIRNAME/../.." && pwd)"
export PATH="$REPO_ROOT/target/release:$PATH"

setup_puff_env() {
  export PUFF_CONFIG_PATH
  PUFF_CONFIG_PATH="$(mktemp -d)"
  export PROJECT_DIR
  PROJECT_DIR="$(mktemp -d)"
  cd "$PROJECT_DIR" || return 1
}

teardown_puff_env() {
  rm -rf "$PUFF_CONFIG_PATH" "$PROJECT_DIR"
}

# Initializes current directory as a puff project with the given name.
puff_init() {
  local name="${1:-myproject}"
  echo "$name" | puff init
}

assert_success() {
  if [ "$status" -ne 0 ]; then
    echo "Expected success but got exit code $status" >&2
    echo "Output: $output" >&2
    return 1
  fi
}

assert_failure() {
  if [ "$status" -eq 0 ]; then
    echo "Expected failure but got success" >&2
    echo "Output: $output" >&2
    return 1
  fi
}

assert_file_exists() {
  if [ ! -e "$1" ]; then
    echo "Expected path to exist: $1" >&2
    return 1
  fi
}

assert_not_exists() {
  if [ -e "$1" ]; then
    echo "Expected path to not exist: $1" >&2
    return 1
  fi
}

assert_symlink() {
  if [ ! -L "$1" ]; then
    echo "Expected symlink at: $1" >&2
    return 1
  fi
}

assert_not_symlink() {
  if [ -L "$1" ]; then
    echo "Expected regular file (not symlink) at: $1" >&2
    return 1
  fi
}

assert_file_content() {
  local actual
  actual=$(cat "$1" 2>/dev/null)
  if [ "$actual" != "$2" ]; then
    echo "File content mismatch in $1" >&2
    echo "Expected: $2" >&2
    echo "Actual:   $actual" >&2
    return 1
  fi
}

assert_output_contains() {
  if ! echo "$output" | grep -qF "$1"; then
    echo "Expected output to contain: $1" >&2
    echo "Actual output: $output" >&2
    return 1
  fi
}

assert_output_not_contains() {
  if echo "$output" | grep -qF "$1"; then
    echo "Expected output to NOT contain: $1" >&2
    echo "Actual output: $output" >&2
    return 1
  fi
}
