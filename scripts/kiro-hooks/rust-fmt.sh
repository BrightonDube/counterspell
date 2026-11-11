#!/usr/bin/env bash
set -euo pipefail

# Hooks receive event JSON on stdin. Keep it consumed even when the path is supplied.
event=$(cat)
file_path=${1:-}
if [[ -z "$file_path" ]] && command -v jq >/dev/null 2>&1; then
  file_path=$(printf '%s' "$event" | jq -r '.filePath // .path // ""' 2>/dev/null || true)
fi

if [[ -z "$file_path" ]]; then
  printf '%s\n' 'skipped: no file path was supplied'
  exit 0
fi
if [[ ! -f "$file_path" ]]; then
  printf 'skipped: file does not exist: %s\n' "$file_path"
  exit 0
fi
if ! command -v rustfmt >/dev/null 2>&1; then
  printf '%s\n' 'skipped: rustfmt is not installed'
  exit 0
fi

rustfmt --edition 2021 -- "$file_path"
