REPO_ROOT="$(cd "$BATS_TEST_DIRNAME/../.." && pwd)"
SCRIPTS_DIR="$REPO_ROOT/scripts"

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

assert_output_contains() {
  if ! echo "$output" | grep -qF "$1"; then
    echo "Expected output to contain: $1" >&2
    echo "Actual output: $output" >&2
    return 1
  fi
}

assert_line() {
  local n="$1" expected="$2"
  local actual
  actual=$(echo "$output" | sed -n "$((n + 1))p")
  if [ "$actual" != "$expected" ]; then
    echo "Expected line $n: $expected" >&2
    echo "Actual line $n:   $actual" >&2
    return 1
  fi
}
