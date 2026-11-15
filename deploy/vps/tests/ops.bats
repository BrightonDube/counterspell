#!/usr/bin/env bats
# Tests for the ops report and the cs_ops_ro grants (deploy-ops T17, R9.4 to R9.6, P11). psql is a stub
# that returns a fixture JSON object. No real database is used.

OPS_DIR="$BATS_TEST_DIRNAME/../ops"
REPORT_SCRIPT="$OPS_DIR/counterspell-ops-report"
REPORT_SQL="$OPS_DIR/sql/ops-report.sql"
ROLE_SQL="$OPS_DIR/sql/ops-role.sql"
ALLOWED_KEYS='["queued_jobs","oldest_queued_age_seconds","expired_leases","dead_jobs_24h","dead_jobs_1h","internal_errors_1h","p95_seconds_1h"]'

HEALTHY='{"queued_jobs":2,"oldest_queued_age_seconds":5,"expired_leases":0,"dead_jobs_24h":0,"dead_jobs_1h":0,"internal_errors_1h":0,"p95_seconds_1h":1.5}'

setup() {
  TEST_ROOT="$(mktemp -d)"
  export STUB_LOG="$TEST_ROOT/calls.log"
  export LOGGER_LOG="$TEST_ROOT/logger.log"
  : > "$STUB_LOG"
  : > "$LOGGER_LOG"
  export PATH="$BATS_TEST_DIRNAME/stubs:$PATH"
  export OPS_DATABASE_URL="postgres://cs_ops_ro@fixture-host/counterspell"
  export OPS_SQL_FILE="$REPORT_SQL"
  export HC_OPS_URL="https://hc.example/ops-ok"
  export HC_OPS_FAIL_URL="https://hc.example/ops-fail"
  export FIXTURE_REPORT="$HEALTHY"
}

teardown() {
  rm -rf "$TEST_ROOT"
}

with_report() {
  export FIXTURE_REPORT="$1"
}

@test "a healthy report exits 0 and pings success only (R9.6)" {
  run bash "$REPORT_SCRIPT"
  [ "$status" -eq 0 ]
  grep -q "curl .*ops-ok" "$STUB_LOG"
  ! grep -q "ops-fail" "$STUB_LOG"
}

@test "a queued job older than 120 seconds signals failure (R9.6)" {
  with_report '{"queued_jobs":4,"oldest_queued_age_seconds":130,"expired_leases":0,"dead_jobs_24h":0,"dead_jobs_1h":0,"internal_errors_1h":0,"p95_seconds_1h":2}'
  run bash "$REPORT_SCRIPT"
  [ "$status" -ne 0 ]
  grep -q "ops-fail" "$STUB_LOG"
  ! grep -q "ops-ok" "$STUB_LOG"
}

@test "an internal error in the last hour signals failure (R9.6)" {
  with_report '{"queued_jobs":0,"oldest_queued_age_seconds":0,"expired_leases":0,"dead_jobs_24h":0,"dead_jobs_1h":0,"internal_errors_1h":1,"p95_seconds_1h":2}'
  run bash "$REPORT_SCRIPT"
  [ "$status" -ne 0 ]
  grep -q "ops-fail" "$STUB_LOG"
}

@test "a dead job in the last hour signals failure (R9.6)" {
  with_report '{"queued_jobs":0,"oldest_queued_age_seconds":0,"expired_leases":0,"dead_jobs_24h":1,"dead_jobs_1h":1,"internal_errors_1h":0,"p95_seconds_1h":2}'
  run bash "$REPORT_SCRIPT"
  [ "$status" -ne 0 ]
  grep -q "ops-fail" "$STUB_LOG"
}

@test "dead jobs older than an hour do not signal failure, since the alert window is one hour (R9.6)" {
  with_report '{"queued_jobs":0,"oldest_queued_age_seconds":0,"expired_leases":0,"dead_jobs_24h":3,"dead_jobs_1h":0,"internal_errors_1h":0,"p95_seconds_1h":2}'
  run bash "$REPORT_SCRIPT"
  [ "$status" -eq 0 ]
  grep -q "ops-ok" "$STUB_LOG"
}

@test "a failed query signals failure and never reports success" {
  export FAIL_PSQL=1
  run bash "$REPORT_SCRIPT"
  [ "$status" -ne 0 ]
  grep -q "ops-fail" "$STUB_LOG"
  ! grep -q "ops-ok" "$STUB_LOG"
}

@test "the journald line holds only the allowed count keys, no other content (R9.4)" {
  run bash "$REPORT_SCRIPT"
  [ "$status" -eq 0 ]
  run jq -e --argjson allowed "$ALLOWED_KEYS" \
    'keys | all(. as $k | $allowed | index($k) != null)' <<< "$(cat "$LOGGER_LOG")"
  [ "$status" -eq 0 ]
}

@test "the report query reads no code, payload, stdout or stderr column (R9.4)" {
  ! grep -Eiq '\b(code|payload|stdout|stderr)\b' "$REPORT_SQL"
}

@test "the grants name exactly the columns in R9.4 and nothing else" {
  grep -Fxq 'GRANT SELECT (state, run_after, lease_expires_at, updated_at) ON jobs TO cs_ops_ro;' "$ROLE_SQL"
  grep -Fxq 'GRANT SELECT (id, status, created_at, finished_at) ON submissions TO cs_ops_ro;' "$ROLE_SQL"
  run grep -c '^GRANT SELECT' "$ROLE_SQL"
  [ "$output" -eq 2 ]
  ! grep -Eiq 'GRANT ALL|TO PUBLIC|SELECT ON (jobs|submissions|run_events)[ ;]' "$ROLE_SQL"
}
