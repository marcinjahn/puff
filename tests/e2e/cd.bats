#!/usr/bin/env bats
load helpers

setup() { setup_puff_env; }
teardown() { teardown_puff_env; }

@test "cd --print: prints the projects data path" {
  run puff cd --print
  assert_success
  assert_output_contains "$PUFF_DATA_PATH"
  assert_output_contains "projects"
}

@test "cd -p: short flag works" {
  run puff cd -p
  assert_success
  assert_output_contains "$PUFF_DATA_PATH"
}

@test "cd: spawns shell in projects directory" {
  # Create a fake shell that just prints its working directory
  local fake_shell="$PUFF_CONFIG_PATH/fake_shell"
  printf '#!/usr/bin/env bash\npwd\n' >"$fake_shell"
  chmod +x "$fake_shell"

  export SHELL="$fake_shell"
  run puff cd
  assert_success
  assert_output_contains "$PUFF_DATA_PATH"
}

@test "cd: sets PUFF_SUBSHELL env var" {
  local fake_shell="$PUFF_CONFIG_PATH/fake_shell"
  printf '#!/usr/bin/env bash\necho "PUFF_SUBSHELL=$PUFF_SUBSHELL"\n' >"$fake_shell"
  chmod +x "$fake_shell"

  export SHELL="$fake_shell"
  run puff cd
  assert_success
  assert_output_contains "PUFF_SUBSHELL=1"
}

@test "cd: exports PUFF_CONFIG_PATH and PUFF_DATA_PATH" {
  local fake_shell="$PUFF_CONFIG_PATH/fake_shell"
  printf '#!/usr/bin/env bash\necho "config=$PUFF_CONFIG_PATH"\necho "data=$PUFF_DATA_PATH"\n' >"$fake_shell"
  chmod +x "$fake_shell"

  export SHELL="$fake_shell"
  run puff cd
  assert_success
  assert_output_contains "config=$PUFF_CONFIG_PATH"
  assert_output_contains "data=$PUFF_DATA_PATH"
}
