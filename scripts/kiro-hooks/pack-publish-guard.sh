#!/usr/bin/env bash
# PreToolUse guard for pack files (pack-authoring-pipeline PA16; PIP-1, PIP-28, PIP-43; P1).
#
# Blocks an agent write that sets `status = "published"` in a pack's meta.toml, and any write
# to an approval.toml. Only Brighton publishes a pack and writes its approval. Matching works on
# the path field, not the whole event, so a document that merely mentions approval.toml is
# allowed. Exit 2 blocks the action and prints the reason. Exit 0 allows it.
set -euo pipefail

event=$(cat)

# A JSON path field such as "path", "filePath" or "target", and the start of its value.
path_field='"(path|filePath|target)"[[:space:]]*:[[:space:]]*"[^"]*'

if grep -Eq -- "${path_field}approval\\.toml\"" <<< "$event"; then
  printf '%s\n' 'blocked: approval.toml is written by Brighton only (PIP-28)' >&2
  exit 2
fi

if grep -Eq -- "${path_field}(packs|pack-assembly)/[^\"]*meta\\.toml\"" <<< "$event" \
  && grep -Eq -- 'status[[:space:]]*=[[:space:]]*\\?"published\\?"' <<< "$event"; then
  printf '%s\n' 'blocked: only Brighton sets status to published, in the approval commit (PIP-1, PIP-28)' >&2
  exit 2
fi

exit 0
