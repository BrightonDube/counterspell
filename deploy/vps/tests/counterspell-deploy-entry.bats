#!/usr/bin/env bats
# Tests for the forced-command entry point (deploy-ops T04, G7, R7.2, R7.9). The sudo stub records
# the exact arguments the entry point would pass to the root script.

ENTRY="$BATS_TEST_DIRNAME/../counterspell-deploy-entry"
HEX_SHA="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
DIG_SERVER="sha256:$(printf 'b%.0s' {1..64})"
DIG_JUDGE="sha256:$(printf 'd%.0s' {1..64})"
DIG_MIGRATE="sha256:$(printf 'e%.0s' {1..64})"

setup() {
  TEST_ROOT="$(mktemp -d)"
  export STUB_LOG="$TEST_ROOT/calls.log"
  : > "$STUB_LOG"
  export PATH="$BATS_TEST_DIRNAME/stubs:$PATH"
}

teardown() {
  rm -rf "$TEST_ROOT"
}

run_entry() {
  export SSH_ORIGINAL_COMMAND="$1"
  run bash "$ENTRY"
}

@test "a valid deploy request reaches the root script with fixed arguments only" {
  run_entry "deploy $HEX_SHA $DIG_SERVER $DIG_JUDGE $DIG_MIGRATE"
  [ "$status" -eq 0 ]
  grep -qx "sudo -n /usr/local/sbin/counterspell-deploy deploy $HEX_SHA $DIG_SERVER $DIG_JUDGE $DIG_MIGRATE" "$STUB_LOG"
}

@test "rollback and status reach the root script with that verb only" {
  run_entry "rollback"
  grep -qx "sudo -n /usr/local/sbin/counterspell-deploy rollback" "$STUB_LOG"
  run_entry "status"
  grep -qx "sudo -n /usr/local/sbin/counterspell-deploy status" "$STUB_LOG"
}

@test "a shell command, even one with a valid verb in front, never reaches sudo" {
  run_entry "status; id"
  [ "$status" -eq 2 ]
  run_entry "deploy $HEX_SHA $DIG_SERVER $DIG_JUDGE $DIG_MIGRATE; id"
  [ "$status" -eq 2 ]
  [ ! -s "$STUB_LOG" ]
}

@test "a deploy with a short SHA or a tag instead of a digest is refused" {
  run_entry "deploy aaaa $DIG_SERVER $DIG_JUDGE $DIG_MIGRATE"
  [ "$status" -eq 2 ]
  run_entry "deploy $HEX_SHA latest $DIG_JUDGE $DIG_MIGRATE"
  [ "$status" -eq 2 ]
  [ ! -s "$STUB_LOG" ]
}

@test "rollback with an extra argument is refused" {
  run_entry "rollback now"
  [ "$status" -eq 2 ]
  [ ! -s "$STUB_LOG" ]
}
