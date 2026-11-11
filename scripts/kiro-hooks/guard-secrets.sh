#!/usr/bin/env bash
set -euo pipefail

mode=${1:-write}
event=$(cat)
paths=''
if command -v jq >/dev/null 2>&1; then
  paths=$(printf '%s' "$event" | jq -r '.. | objects | to_entries[]? | select(.key | test("^(path|filePath|target|uri)$"; "i")) | .value | strings' 2>/dev/null || true)
fi
if [[ -z "$paths" ]]; then
  paths=$event
  # Keep the safety check useful on systems without jq. The exact `.env`
  # form is matched; `.env.example` has a dot after `env` and is allowed.
  if grep -Eiq '(^|["/ ])\.env(["/ ,}]|$)' <<< "$event"; then
    printf '%s\n' "blocked: refusing $mode access to secret-like .env path"
    exit 2
  fi
  if grep -Eiq '(^|["/ ])(id_rsa[^"/ ,}]*|[^"/ ,}]*\.pem)(["/ ,}]|$)|(["/ ])secrets/' <<< "$event"; then
    printf '%s\n' "blocked: refusing $mode access to a key or secrets path"
    exit 2
  fi
fi

while IFS= read -r path; do
  [[ -z "$path" ]] && continue
  if [[ "$path" == '.env' || "$path" == */.env || "$path" == .env.* || "$path" == */.env.* ]]; then
    if [[ "$path" != '.env.example' && "$path" != */.env.example ]]; then
      printf 'blocked: refusing %s access to secret-like file %s\n' "$mode" "$path"
      exit 2
    fi
  fi
  if [[ "$path" == *.pem || "$path" == *id_rsa* || "$path" == secrets/* || "$path" == */secrets/* ]]; then
    printf 'blocked: refusing %s access to secret-like file %s\n' "$mode" "$path"
    exit 2
  fi
done <<< "$paths"

if [[ "$mode" == write ]]; then
  if grep -Eiq 'sk-[A-Za-z0-9]{20,}|gsk_[A-Za-z0-9_-]{20,}|re_[A-Za-z0-9_-]{20,}' <<< "$event"; then
    printf '%s\n' 'blocked: probable API key detected in write input'
    exit 2
  fi
fi

exit 0
