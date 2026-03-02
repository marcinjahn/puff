#!/usr/bin/env bats
load helpers

setup() { setup_puff_env; }
teardown() { teardown_puff_env; }

@test "cd --print: prints the projects data path" {
  run puff cd --print
  assert_success
  output="$(echo "$output" | normalize_output)"
  assert_output_contains "$(native_path "$PUFF_DATA_PATH")"
  assert_output_contains "projects"
}

@test "cd -p: short flag works" {
  run puff cd -p
  assert_success
  output="$(echo "$output" | normalize_output)"
  assert_output_contains "$(native_path "$PUFF_DATA_PATH")"
}

@test "cd: spawns shell in projects directory" {
  if is_windows; then
    local fake_shell="$PUFF_CONFIG_PATH/fake_shell.bat"
    echo '@cd' >"$fake_shell"
    export COMSPEC="$(native_path "$fake_shell")"
  else
    local fake_shell="$PUFF_CONFIG_PATH/fake_shell"
    printf '#!/usr/bin/env bash\npwd\n' >"$fake_shell"
    chmod +x "$fake_shell"
    export SHELL="$fake_shell"
  fi

  run puff cd
  assert_success
  output="$(echo "$output" | normalize_output)"
  assert_output_contains "$(native_path "$PUFF_DATA_PATH")"
}

@test "cd: sets PUFF_SUBSHELL env var" {
  if is_windows; then
    local fake_shell="$PUFF_CONFIG_PATH/fake_shell.bat"
    echo '@echo PUFF_SUBSHELL=%PUFF_SUBSHELL%' >"$fake_shell"
    export COMSPEC="$(native_path "$fake_shell")"
  else
    local fake_shell="$PUFF_CONFIG_PATH/fake_shell"
    printf '#!/usr/bin/env bash\necho "PUFF_SUBSHELL=$PUFF_SUBSHELL"\n' >"$fake_shell"
    chmod +x "$fake_shell"
    export SHELL="$fake_shell"
  fi

  run puff cd
  assert_success
  output="$(echo "$output" | normalize_output)"
  assert_output_contains "PUFF_SUBSHELL=1"
}

@test "cd: exports PUFF_CONFIG_PATH and PUFF_DATA_PATH" {
  if is_windows; then
    local fake_shell="$PUFF_CONFIG_PATH/fake_shell.bat"
    cat >"$fake_shell" <<'BATCH'
@echo config=%PUFF_CONFIG_PATH%
@echo data=%PUFF_DATA_PATH%
BATCH
    export COMSPEC="$(native_path "$fake_shell")"
  else
    local fake_shell="$PUFF_CONFIG_PATH/fake_shell"
    printf '#!/usr/bin/env bash\necho "config=$PUFF_CONFIG_PATH"\necho "data=$PUFF_DATA_PATH"\n' >"$fake_shell"
    chmod +x "$fake_shell"
    export SHELL="$fake_shell"
  fi

  run puff cd
  assert_success
  output="$(echo "$output" | normalize_output)"
  assert_output_contains "config=$(native_path "$PUFF_CONFIG_PATH")"
  assert_output_contains "data=$(native_path "$PUFF_DATA_PATH")"
}
