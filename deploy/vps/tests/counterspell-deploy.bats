#!/usr/bin/env bats
# Tests for deploy/vps/counterspell-deploy (deploy-ops T03, requirements R7, P3 to P7).
# Every external command is a stub in tests/stubs/ that records its arguments.
# No test contacts a registry, a host or a real service.

HEX_SHA="aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa"
OLD_SHA="cccccccccccccccccccccccccccccccccccccccc"
DIG_SERVER="sha256:$(printf 'b%.0s' {1..64})"
DIG_JUDGE="sha256:$(printf 'd%.0s' {1..64})"
DIG_MIGRATE="sha256:$(printf 'e%.0s' {1..64})"
OLD_DIG="sha256:$(printf 'f%.0s' {1..64})"

setup() {
  TEST_ROOT="$(mktemp -d)"
  export COUNTERSPELL_DIR="$TEST_ROOT/opt"
  export COUNTERSPELL_HEALTH_URL="https://counterspell.example/api/health"
  export STUB_LOG="$TEST_ROOT/calls.log"
  export LOGGER_LOG="$TEST_ROOT/logger.log"
  : > "$STUB_LOG"
  : > "$LOGGER_LOG"
  export PATH="$BATS_TEST_DIRNAME/stubs:$PATH"
  mkdir -p "$COUNTERSPELL_DIR"
  printf 'services: {}\n' > "$COUNTERSPELL_DIR/docker-compose.yml"
  printf 'services: {}\n' > "$COUNTERSPELL_DIR/compose.prod.yml"
  printf 'counterspell.example {}\n' > "$COUNTERSPELL_DIR/Caddyfile"
  (cd "$COUNTERSPELL_DIR" && sha256sum docker-compose.yml compose.prod.yml Caddyfile > compose.sha256)
}

teardown() {
  rm -rf "$TEST_ROOT"
}

# Writes a current release record for the old SHA, and a host.env copy with host-only keys.
write_current_release() {
  cat > "$COUNTERSPELL_DIR/deploy.env" <<EOF
COUNTERSPELL_SHA=$OLD_SHA
COUNTERSPELL_SERVER_IMAGE=ghcr.io/brightondube/counterspell-server@$OLD_DIG
COUNTERSPELL_JUDGE_IMAGE=ghcr.io/brightondube/counterspell-judge@$OLD_DIG
COUNTERSPELL_MIGRATE_IMAGE=ghcr.io/brightondube/counterspell-migrate@$OLD_DIG
CADDY_SITE_ADDRESS=counterspell.example
EOF
}

run_request() {
  export SSH_ORIGINAL_COMMAND="$1"
  run bash "$BATS_TEST_DIRNAME/../counterspell-deploy"
}

deploy_request() {
  echo "deploy $HEX_SHA $DIG_SERVER $DIG_JUDGE $DIG_MIGRATE"
}

stub_calls() {
  grep -v '^logger' "$STUB_LOG" || true
}

@test "a request that is not deploy, rollback or status is refused and no tool runs" {
  run_request "rm -rf /"
  [ "$status" -eq 2 ]
  [ -z "$(stub_calls)" ]
}

@test "a deploy with a malformed digest is refused before any tool runs" {
  run_request "deploy $HEX_SHA sha256:notahexdigest $DIG_JUDGE $DIG_MIGRATE"
  [ "$status" -eq 2 ]
  [ -z "$(stub_calls)" ]
}

@test "a failing migrate stops the deploy before the stack is started (G1, R7.3, R7.4, P3)" {
  export FAIL_MIGRATE=1
  run_request "$(deploy_request)"
  [ "$status" -ne 0 ]
  ! grep -q ' up -d' "$STUB_LOG"
}

@test "a failing readiness check restores the previous release and keeps the candidate out of deploy.env (R7.7, P6)" {
  write_current_release
  export FAIL_CURL=1
  run_request "$(deploy_request)"
  [ "$status" -ne 0 ]
  grep -q "^COUNTERSPELL_SHA=$OLD_SHA$" "$COUNTERSPELL_DIR/deploy.env"
  ! grep -q "$HEX_SHA" "$COUNTERSPELL_DIR/deploy.env"
}

@test "rollback swaps the release records and runs no migration (R7.9, P4)" {
  write_current_release
  printf 'COUNTERSPELL_SHA=%s\n' "$HEX_SHA" > "$COUNTERSPELL_DIR/deploy.env.previous"
  run_request "rollback"
  [ "$status" -eq 0 ]
  grep -q "^COUNTERSPELL_SHA=$HEX_SHA$" "$COUNTERSPELL_DIR/deploy.env"
  grep -q "^COUNTERSPELL_SHA=$OLD_SHA$" "$COUNTERSPELL_DIR/deploy.env.previous"
  ! grep -q 'run --rm migrate' "$STUB_LOG"
}

@test "rollback refuses when there is no previous release record (R7.9, P4)" {
  write_current_release
  run_request "rollback"
  [ "$status" -ne 0 ]
  ! grep -q 'docker' "$STUB_LOG"
}

@test "a second deploy while the lock is held is refused (R7.11, P5)" {
  export FAIL_FLOCK=1
  run_request "$(deploy_request)"
  [ "$status" -eq 2 ]
  ! grep -q 'docker' "$STUB_LOG"
}

@test "a compose file that differs from compose.sha256 is refused before any pull (R7.13, P7)" {
  printf 'services: { tampered: {} }\n' > "$COUNTERSPELL_DIR/compose.prod.yml"
  run_request "$(deploy_request)"
  [ "$status" -ne 0 ]
  ! grep -q ' pull ' "$STUB_LOG"
}

@test "status prints only the SHA and the digests, never host values (R7.14)" {
  write_current_release
  run_request "status"
  [ "$status" -eq 0 ]
  [[ "$output" == *"COUNTERSPELL_SHA=$OLD_SHA"* ]]
  [[ "$output" != *"CADDY_SITE_ADDRESS"* ]]
  [[ "$output" != *"counterspell.example"* ]]
}
