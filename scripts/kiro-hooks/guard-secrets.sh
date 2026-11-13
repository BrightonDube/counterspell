#!/usr/bin/env bash
set -euo pipefail

mode=${1:-write}
event=$(cat)
paths=''

say_block() {
  # Print to stdout and stderr so both CLI and hook log capture the reason.
  printf '%s\n' "$*" | tee /dev/stderr
  exit 2
}

if command -v jq >/dev/null 2>&1; then
  paths=$(printf '%s' "$event" | jq -r '.. | objects | to_entries[]? | select(.key | test("^(path|filePath|target|uri)$"; "i")) | .value | strings' 2>/dev/null || true)
fi
if [[ -z "$paths" ]]; then
  paths=$event
  # Keep the safety check useful on systems without jq. The exact `.env`
  # form is matched; `.env.example` has a dot after `env` and is allowed.
  if grep -Eiq '(^|["/ ])\.env(["/ ,}]|$)' <<< "$event"; then
    say_block "blocked: refusing $mode access to secret-like .env path"
  fi
  if grep -Eiq '(^|["/ ])(id_rsa[^"/ ,}]*|[^"/ ,}]*\.pem)(["/ ,}]|$)|(["/ ])secrets/' <<< "$event"; then
    say_block "blocked: refusing $mode access to a key or secrets path"
  fi
fi

while IFS= read -r path; do
  [[ -z "$path" ]] && continue
  if [[ "$path" == '.env' || "$path" == */.env || "$path" == .env.* || "$path" == */.env.* ]]; then
    if [[ "$path" != '.env.example' && "$path" != */.env.example ]]; then
      say_block "blocked: refusing $mode access to secret-like file $path"
    fi
  fi
  if [[ "$path" == *.pem || "$path" == *id_rsa* || "$path" == secrets/* || "$path" == */secrets/* ]]; then
    say_block "blocked: refusing $mode access to secret-like file $path"
  fi
done <<< "$paths"

if [[ "$mode" == write ]]; then
  # Token-boundary anchors: otherwise ordinary identifiers such as
  # failure_reason_public_messages_are_distinct match re_[A-Za-z0-9_-]{20,}.
  if grep -Eiq '(^|[^A-Za-z0-9_])sk-[A-Za-z0-9]{20,}' <<< "$event"; then
    say_block 'blocked: probable API key detected in write input (sk-...)'
  fi
  if grep -Eiq '(^|[^A-Za-z0-9_])gsk_[A-Za-z0-9_-]{20,}' <<< "$event"; then
    say_block 'blocked: probable API key detected in write input (gsk_...)'
  fi
  if grep -Eiq '(^|[^A-Za-z0-9_])re_[A-Za-z0-9_-]{20,}' <<< "$event"; then
    say_block 'blocked: probable API key detected in write input (re_...)'
  fi
fi

exit 0
