#!/usr/bin/env bash
# Regression checks for scripts/kiro-hooks/guard-secrets.sh.
# Run from the repository root: bash scripts/kiro-hooks/test-guard-secrets.sh
set -euo pipefail
cd "$(dirname "$0")/../.."
HOOK=scripts/kiro-hooks/guard-secrets.sh
fail=0

# Build token-like strings at runtime so this file does not contain a full key.
re_key="re_$(printf 'a%.0s' {1..24})0123"
sk_key="sk-$(printf 'a%.0s' {1..24})0123"
gsk_key="gsk_$(printf 'a%.0s' {1..24})0123"
ask_word="ask-$(printf 'a%.0s' {1..24})0123"

expect() {
  local want=$1 name=$2 data=$3
  local out rc
  set +e
  out=$(printf '%s' "$data" | bash "$HOOK" write 2>&1)
  rc=$?
  set -e
  if [[ "$rc" -ne "$want" ]]; then
    printf 'FAIL %s: exit %s want %s out=%s\n' "$name" "$rc" "$want" "$out"
    fail=1
  elif [[ "$want" -eq 2 && "$out" != blocked:* ]]; then
    printf 'FAIL %s: blocked without a blocked: reason out=%s\n' "$name" "$out"
    fail=1
  else
    printf 'ok  %s\n' "$name"
  fi
}

expect 2 "block .env" '{"path":".env","content":"DATABASE_URL=hidden"}'
expect 0 "allow .env.example" '{"path":".env.example","content":"DATABASE_URL=postgres://localhost/dev"}'
expect 2 "block pem path" '{"path":"certs/server.pem","content":"x"}'
expect 2 "block id_rsa path" '{"path":"id_rsa","content":"x"}'
expect 2 "block secrets/ path" '{"path":"secrets/config.toml","content":"x"}'
expect 2 "block re_ key" "const KEY: \&str = \"${re_key}\";"
expect 2 "block sk- key" "const KEY: \&str = \"${sk_key}\";"
expect 2 "block gsk_ key" "const KEY: \&str = \"${gsk_key}\";"
expect 0 "allow failure_reason_public_messages_are_distinct" 'fn failure_reason_public_messages_are_distinct() {}'
expect 0 "allow failure_reason_round_trip_and_distinct_messages" 'fn failure_reason_round_trip_and_distinct_messages() {}'
expect 0 "allow ask- long word" "fn task() { let ${ask_word} = 1; }"
expect 0 "allow clean rust" 'fn can_transition() { assert!(true); }'

if [[ "$fail" -ne 0 ]]; then
  echo "guard-secrets tests failed"
  exit 1
fi
echo "all guard-secrets tests passed"
