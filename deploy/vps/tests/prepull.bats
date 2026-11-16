#!/usr/bin/env bats
# Tests for deploy/vps/prepull.sh (deploy-ops T13, R3.6, R3.7). docker is a stub. The catalogue is a
# fixture with fake digests, so no image is ever pulled.

PREPULL="$BATS_TEST_DIRNAME/../prepull.sh"
FIXTURE_TOML="$BATS_TEST_DIRNAME/fixtures/languages.toml"

setup() {
  TEST_ROOT="$(mktemp -d)"
  export STUB_LOG="$TEST_ROOT/calls.log"
  : > "$STUB_LOG"
  export PATH="$BATS_TEST_DIRNAME/stubs:$PATH"
  export LANGUAGES_TOML="$FIXTURE_TOML"
  export SANDBOX_DOCKER_SOCKET="/run/user/1001/docker.sock"
}

teardown() {
  rm -rf "$TEST_ROOT"
}

@test "pulls only the enabled languages, each by its digest, on the sandbox socket (R3.6, R3.7)" {
  run bash "$PREPULL"
  [ "$status" -eq 0 ]
  grep -q "docker -H unix:///run/user/1001/docker.sock pull python:fixture@sha256:aaaa" "$STUB_LOG"
  grep -q "pull node:fixture@sha256:bbbb" "$STUB_LOG"
  ! grep -q "eclipse-temurin" "$STUB_LOG"
}

@test "never touches the main daemon socket" {
  run bash "$PREPULL"
  [ "$status" -eq 0 ]
  [ -z "$(grep 'docker.sock' "$STUB_LOG" | grep -v 'unix:///run/user/1001/docker.sock' || true)" ]
  ! grep -q "/var/run/docker.sock" "$STUB_LOG"
}

@test "refuses an enabled image that has a tag but no digest, and pulls nothing" {
  cat > "$TEST_ROOT/tags.toml" <<'EOF'
[python]
enabled = true
image = "python:3.13-slim"
EOF
  export LANGUAGES_TOML="$TEST_ROOT/tags.toml"
  run bash "$PREPULL"
  [ "$status" -ne 0 ]
  ! grep -q " pull " "$STUB_LOG"
}

@test "a local digest that differs from the pinned one fails the check (R3.6)" {
  export FAKE_REPO_DIGESTS='["python:fixture@sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd"]'
  run bash "$PREPULL"
  [ "$status" -ne 0 ]
  [[ "$output" == *"digest mismatch"* ]]
}
