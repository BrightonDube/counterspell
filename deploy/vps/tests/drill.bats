#!/usr/bin/env bats
# Tests for the monthly restore drill (deploy-ops T18, R8.8 to R8.10, Q23). Stubs stand in for rclone,
# age and docker. The identity file is a fixture under /dev/shm, removed by the drill's trap.

DRILL="$BATS_TEST_DIRNAME/../restore-drill/counterspell-restore-drill"
TEMPLATE="$BATS_TEST_DIRNAME/../restore-drills/TEMPLATE.md"
NAME="counterspell-20261115T013000Z.dump.age"

setup() {
  TEST_ROOT="$(mktemp -d)"
  export STUB_LOG="$TEST_ROOT/calls.log"
  export LOGGER_LOG="$TEST_ROOT/logger.log"
  export PATH="$BATS_TEST_DIRNAME/stubs:$PATH"
  : > "$STUB_LOG"
  export FIXTURE_DIR="$TEST_ROOT/fixtures"
  export DRILL_RESULT_DIR="$TEST_ROOT/results"
  export DRILL_TEMPLATE="$TEMPLATE"
  export BACKUP_BUCKET="fixture-remote:counterspell-backups"
  export RCLONE_CONFIG="$TEST_ROOT/rclone.conf"
  export DRILL_NET="fixture_data"
  export FIXTURE_LSF="$NAME"
  mkdir -p "$FIXTURE_DIR"

  # A fixture dump is base64 text, matching the age stub. The manifest holds its SHA-256 and the counts.
  printf 'fixture-dump-bytes\n' | base64 > "$FIXTURE_DIR/$NAME"
  local sha
  sha="$(sha256sum "$FIXTURE_DIR/$NAME" | cut -d' ' -f1)"
  write_manifest "$sha" 5
  export DRILL_IDENTITY_FILE
  DRILL_IDENTITY_FILE="$(mktemp /dev/shm/cs-drill-test.XXXXXX)"
  export TEST_IDENTITY="$DRILL_IDENTITY_FILE"
  printf 'fixture-identity-not-a-secret\n' > "$DRILL_IDENTITY_FILE"
}

teardown() {
  rm -f -- "$TEST_IDENTITY" 2>/dev/null || true
  rm -rf "$TEST_ROOT"
}

# Writes the manifest with the given sha and users count; other counts match the docker stub.
write_manifest() {
  cat > "$FIXTURE_DIR/counterspell-20261115T013000Z.manifest" <<EOF
$1  $NAME
users $2
submissions 9
jobs 4
run_events 30
migration 3
EOF
}

@test "a matching drill passes, writes a counts-only record, and removes the identity and the container (R8.8, R8.9, Q23)" {
  run bash "$DRILL"
  [ "$status" -eq 0 ]
  result="$DRILL_RESULT_DIR/$(date -u +%Y-%m).md"
  [ -f "$result" ]
  grep -q '| Result | PASS |' "$result"
  grep -q "$NAME" "$result"
  ! grep -qi 'password' "$result"
  [ ! -e "$DRILL_IDENTITY_FILE" ]
  grep -q 'docker rm -f cs-drill-' "$STUB_LOG"
}

@test "a checksum mismatch stops the drill before any container starts and still removes the identity (R8.8)" {
  printf 'tampered\n' > "$FIXTURE_DIR/$NAME"
  run bash "$DRILL"
  [ "$status" -ne 0 ]
  ! grep -q 'docker run' "$STUB_LOG"
  [ ! -e "$DRILL_IDENTITY_FILE" ]
}

@test "a row count that differs from the manifest records FAIL and exits non-zero (R8.9)" {
  write_manifest "$(sha256sum "$FIXTURE_DIR/$NAME" | cut -d' ' -f1)" 999
  run bash "$DRILL"
  [ "$status" -ne 0 ]
  result="$DRILL_RESULT_DIR/$(date -u +%Y-%m).md"
  grep -q '| Result | FAIL |' "$result"
}

@test "an identity path outside /dev/shm is refused and the file is never deleted" {
  local outside="$TEST_ROOT/not-tmpfs-key.txt"
  printf 'keep me\n' > "$outside"
  export DRILL_IDENTITY_FILE="$outside"
  run bash "$DRILL"
  [ "$status" -eq 2 ]
  [ -f "$outside" ]
  ! grep -q 'docker run' "$STUB_LOG"
}
