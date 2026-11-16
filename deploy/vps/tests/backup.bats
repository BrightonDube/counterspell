#!/usr/bin/env bats
# Tests for deploy/vps/backup/counterspell-backup (deploy-ops T16, R8.1 to R8.7, P10). Every external
# command is a stub; no real database, object store, age key or Healthchecks URL is used.

SCRIPT="$BATS_TEST_DIRNAME/../backup/counterspell-backup"
PLAINTEXT_MARKER="PLAINTEXT-MARKER-DO-NOT-STORE"

setup() {
  TEST_ROOT="$(mktemp -d)"
  export COUNTERSPELL_DIR="$TEST_ROOT/opt"
  export BACKUP_DIR="$TEST_ROOT/backups"
  export STUB_LOG="$TEST_ROOT/calls.log"
  export LOGGER_LOG="$TEST_ROOT/logger.log"
  export PATH="$BATS_TEST_DIRNAME/stubs:$PATH"
  : > "$STUB_LOG"
  : > "$LOGGER_LOG"
  mkdir -p "$COUNTERSPELL_DIR" "$BACKUP_DIR/dumps" "$BACKUP_DIR/manifests"
  export AGE_RECIPIENT="age1fixturepublicrecipientonly"
  export BACKUP_BUCKET="fixture-remote:counterspell-backups"
  export RCLONE_CONFIG="$TEST_ROOT/rclone.conf"
  export HC_SUCCESS_URL="https://hc.example/success-fixture"
  export HC_FAIL_URL="https://hc.example/fail-fixture"
}

teardown() {
  rm -rf "$TEST_ROOT"
}

# Creates n older dump files (and their manifests) named so they sort before the run's own file.
seed_old_dumps() {
  local n="$1" i
  for ((i = 1; i <= n; i++)); do
    local name
    name="counterspell-20261101T0130$(printf '%02d' "$i")Z"
    printf 'old\n' > "$BACKUP_DIR/dumps/$name.dump.age"
    printf 'old\n' > "$BACKUP_DIR/manifests/$name.manifest"
  done
}

count_dumps() {
  find "$BACKUP_DIR/dumps" -maxdepth 1 -name 'counterspell-*.dump.age' | wc -l
}

@test "a successful run writes one encrypted dump, a manifest with counts, and pings success (R8.2, R8.4, R8.7)" {
  run bash "$SCRIPT"
  [ "$status" -eq 0 ]
  [ "$(count_dumps)" -eq 1 ]
  manifest="$(ls "$BACKUP_DIR"/manifests/*.manifest)"
  grep -Eq '^[0-9a-f]{64}  counterspell-[0-9TZ]+\.dump\.age$' "$manifest"
  grep -q '^users 5$' "$manifest"
  grep -q '^migration 3$' "$manifest"
  grep -q "curl .*success-fixture" "$STUB_LOG"
  ! grep -q "fail-fixture" "$STUB_LOG"
}

@test "plaintext is never written to disk: the pg_dump output exists only as encrypted bytes (R8.1, P10)" {
  run bash "$SCRIPT"
  [ "$status" -eq 0 ]
  ! grep -rl -- "$PLAINTEXT_MARKER" "$BACKUP_DIR"
  grep -q '^age -r ' "$STUB_LOG"
}

@test "prune keeps exactly the 7 newest dumps, including the one just written (R8.3)" {
  seed_old_dumps 8
  run bash "$SCRIPT"
  [ "$status" -eq 0 ]
  [ "$(count_dumps)" -eq 7 ]
  # The newest old dump survives and the oldest is gone.
  [ -f "$BACKUP_DIR/dumps/counterspell-20261101T013008Z.dump.age" ]
  [ ! -f "$BACKUP_DIR/dumps/counterspell-20261101T013001Z.dump.age" ]
}

@test "a failed age step keeps every older dump and writes no new dump (R8.3)" {
  seed_old_dumps 3
  export FAIL_AGE=1
  run bash "$SCRIPT"
  [ "$status" -ne 0 ]
  [ "$(count_dumps)" -eq 3 ]
  [ -z "$(find "$BACKUP_DIR/dumps" -name '*.partial')" ]
  grep -q "fail-fixture" "$STUB_LOG"
}

@test "a failed upload keeps the local encrypted dump and signals failure (R8.4, R8.7)" {
  export FAIL_RCLONE=1
  run bash "$SCRIPT"
  [ "$status" -ne 0 ]
  [ "$(count_dumps)" -eq 1 ]
  grep -q "fail-fixture" "$STUB_LOG"
  ! grep -q "success-fixture" "$STUB_LOG"
}

@test "a missing age recipient stops before any database call" {
  unset AGE_RECIPIENT
  run bash "$SCRIPT"
  [ "$status" -ne 0 ]
  ! grep -q 'docker' "$STUB_LOG"
  [ "$(count_dumps)" -eq 0 ]
}
