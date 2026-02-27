#!/usr/bin/env bats
load helpers

@test "completions: generates bash completions" {
  run puff completions bash
  assert_success
  assert_output_contains "puff"
}

@test "completions: generates fish completions" {
  run puff completions fish
  assert_success
  assert_output_contains "puff"
}

@test "completions: generates zsh completions" {
  run puff completions zsh
  assert_success
  assert_output_contains "puff"
}

@test "completions: generates powershell completions" {
  run puff completions powershell
  assert_success
  assert_output_contains "puff"
}

@test "completions: fails on unknown shell" {
  run puff completions unknownshell
  assert_failure
}
