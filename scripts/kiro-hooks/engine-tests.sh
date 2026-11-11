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
if [[ ! -d crates/engine ]]; then
  printf '%s\n' 'skipped: crates/engine does not exist'
  exit 0
fi

if cargo test -q -p counterspell-engine 2>&1 | tail -40; then
  exit 0
else
  status=$?
  printf 'cargo test failed with status %s\n' "$status"
  exit "$status"
fi
