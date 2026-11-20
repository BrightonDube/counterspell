#!/usr/bin/env bash
# Verifies the production build keeps the break-room bundles clean:
# 1. no GSAP in any chunk that ships chess code;
# 2. no page node that serves the IDE (monaco) or landing marks a chess
#    chunk as a dependency — so those pages never download chess code.
set -euo pipefail
cd "$(dirname "$0")/../web"
[ -d build ] || { echo "run npm run build first" >&2; exit 1; }
fail=0

chess_chunks=$(grep -rl 'cm-chessboard\|chess\.js' build/_app/immutable || true)
for chunk in $chess_chunks; do
  if grep -q 'gsap' "$chunk"; then
    echo "bundle: gsap found in break chunk $chunk"
    fail=1
  fi
done

for chunk in $chess_chunks; do
  base=$(basename "$chunk")
  for node in $(find build/_app/immutable/nodes -name '*.js'); do
    if grep -q "$base" "$node" \
      && grep -q 'monaco-editor' "$node"; then
      echo "bundle: IDE node $node depends on chess chunk $base"
      fail=1
    fi
  done
done
exit "$fail"
