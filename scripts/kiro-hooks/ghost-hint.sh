#!/usr/bin/env bash
set -euo pipefail

event=$(cat)
# This is deliberately an informational hook. It never blocks a tool call.
if ! grep -Eiq 'counterspell-(engine|grimoire)|pack_[[:alnum:]_-]+' <<< "$event"; then
  exit 0
fi
if ! grep -Eiq 'FAILED|panicked' <<< "$event"; then
  exit 0
fi

printf '%s\n' 'A pack/engine test failed. Consider calling @counterspell/find_ghost_input (it runs through the sandbox) or reading the proptest shrunk case, and explain the minimized counterexample before editing.'
exit 0
