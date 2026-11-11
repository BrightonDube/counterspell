#!/usr/bin/env bash
set -euo pipefail

event=$(cat)
if ! command -v cargo >/dev/null 2>&1; then
  printf '%s\n' 'skipped: cargo is not installed'
  exit 0
fi
if [[ ! -f Cargo.toml ]]; then
  printf '%s\n' 'skipped: Cargo.toml does not exist'
  exit 0
fi
if [[ ! -d crates ]]; then
  printf '%s\n' 'skipped: crates directory does not exist'
  exit 0
fi

tmp_file=$(mktemp)
trap 'rm -f "$tmp_file"' EXIT
if cargo check -q --workspace >"$tmp_file" 2>&1; then
  printf '%s\n' 'pre-task cargo check: PASS'
else
  status=$?
  printf 'pre-task cargo check: FAIL (status %s)\n' "$status"
  sed -n '1,30p' "$tmp_file"
fi
exit 0
