#!/usr/bin/env bash
set -euo pipefail

event=$(cat)
allow_list='MIT;Apache-2.0;BSD-2-Clause;BSD-3-Clause;ISC;Zlib;Unicode-3.0;0BSD;CC0-1.0;MPL-2.0;OFL-1.1'
status=0

if [[ -f Cargo.toml && -d crates ]]; then
  if command -v cargo-deny >/dev/null 2>&1; then
    if cargo deny check licenses; then
      :
    else
      status=$?
    fi
  else
    printf '%s\n' 'skipped: cargo-deny is not installed'
  fi
else
  printf '%s\n' 'skipped: Cargo.toml or crates does not exist'
fi

if [[ -f web/package.json ]]; then
  if command -v npx >/dev/null 2>&1; then
    if (cd web && npx --yes license-checker-rseidelsohn --onlyAllow "$allow_list" --excludePackages gsap); then
      :
    else
      status=$?
    fi
  else
    printf '%s\n' 'skipped: npx is not installed'
  fi
else
  printf '%s\n' 'skipped: web/package.json does not exist'
fi

exit "$status"
