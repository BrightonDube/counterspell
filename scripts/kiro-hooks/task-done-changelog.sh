#!/usr/bin/env bash
set -euo pipefail

event=$(cat)
mkdir -p docs/kiro-log

spec_task=''
verification=''
if command -v jq >/dev/null 2>&1; then
  spec_task=$(printf '%s' "$event" | jq -r '[.. | objects | to_entries[]? | select(.key | test("spec|task"; "i")) | .value | strings] | unique | join(", ")' 2>/dev/null || true)
  verification=$(printf '%s' "$event" | jq -r '[.. | objects | to_entries[]? | select(.key | test("verification|result|check"; "i")) | .value | tostring] | unique | join(", ")' 2>/dev/null || true)
else
  spec_task=$(grep -Eio '"(spec(Id|ification)?|task(Id)?)"[[:space:]]*:[[:space:]]*"[^"]+"' <<< "$event" | sed -E 's/.*:[[:space:]]*"([^"]+)"/\1/' | head -1 || true)
  verification=$(grep -Eio '"(verification|result|checks?)"[[:space:]]*:[[:space:]]*"[^"]+"' <<< "$event" | sed -E 's/.*:[[:space:]]*"([^"]+)"/\1/' | head -1 || true)
fi
[[ -n "$spec_task" ]] || spec_task='not reported'
[[ -n "$verification" ]] || verification='not reported'

if command -v git >/dev/null 2>&1 && git rev-parse --show-toplevel >/dev/null 2>&1; then
  branch=$(git branch --show-current 2>/dev/null || true)
  commit=$(git rev-parse HEAD 2>/dev/null || true)
else
  branch='unknown'
  commit='unknown'
fi
[[ -n "$branch" ]] || branch='detached-or-unknown'
[[ -n "$commit" ]] || commit='unknown'

printf -- '- %s | spec/task: %s | branch: %s | commit: %s | verification: %s\n' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$spec_task" "$branch" "$commit" "$verification" >> docs/kiro-log/tasks.md
