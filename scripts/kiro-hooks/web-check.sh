#!/usr/bin/env bash
set -euo pipefail

event=$(cat)
if [[ ! -d web || ! -f web/package.json ]]; then
  printf '%s\n' 'skipped: web app does not exist'
  exit 0
fi
if ! command -v npx >/dev/null 2>&1; then
  printf '%s\n' 'skipped: npx is not installed'
  exit 0
fi

if (cd web && npx svelte-check --threshold error --output human 2>&1 | tail -30); then
  exit 0
else
  status=$?
  printf 'svelte-check failed with status %s\n' "$status"
  exit "$status"
fi
