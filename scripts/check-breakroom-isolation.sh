#!/usr/bin/env bash
# Fails when anything outside the break-room isolation map imports the feature.
set -euo pipefail
cd "$(dirname "$0")/.."
fail=0

# Rust side: breakroom may only be named in its own module and the lib.rs mount.
while IFS= read -r hit; do
  case "$hit" in
    crates/server/src/breakroom/*|crates/server/src/lib.rs:*|crates/server/tests/breakroom.rs:*) ;;
    *) echo "isolation: unexpected breakroom reference: $hit"; fail=1;;
  esac
done < <(grep -rn 'breakroom' crates/ --include='*.rs' || true)

# Web side: lib/break or the chess packages may only appear under routes/break,
# lib/break, lib/nav/NavEntry.svelte (the one allowed hook), the type shim and
# the notices file.
while IFS= read -r hit; do
  case "$hit" in
    web/src/routes/break/*|web/src/lib/break/*|web/src/lib/nav/NavEntry.svelte:*|web/src/types/cm-chessboard.d.ts:*) ;;
    *) echo "isolation: unexpected chess/break import: $hit"; fail=1;;
  esac
done < <(grep -rn -E "lib/break|chess\.js|cm-chessboard" web/src --include='*.svelte' --include='*.ts' --include='*.js' || true)

# Banned engines must never appear in the lockfile.
if grep -q '"chessground"' web/package-lock.json || grep -q '"stockfish"' web/package-lock.json; then
  echo "isolation: banned chess package in package-lock.json"
  fail=1
fi
exit "$fail"
