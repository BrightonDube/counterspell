#!/usr/bin/env bash
# PreToolUse guard for pack files (pack-authoring-pipeline PA16; PIP-1, PIP-28, PIP-43; P1).
#
# Blocks, before the write happens:
#   - any write to an approval.toml,
#   - a write that sets status to published in a pack's meta.toml,
#   - a write of a private file name (hidden, generator, known ghosts, retired values) under packs/.
# Matching works on the path field, not the whole event, so a document that merely mentions
# approval.toml is allowed. Paths and values are compared in lower case, and the published value
# may use single or double quotes. Exit 2 blocks the action. Exit 0 allows it.
set -euo pipefail

event=$(cat)
lower=$(printf '%s' "$event" | tr '[:upper:]' '[:lower:]')

# A JSON path field such as "path", "filePath" or "target", and the start of its value.
path_field='"(path|filepath|target)"[[:space:]]*:[[:space:]]*"[^"]*'

if grep -Eq -- "${path_field}approval\\.toml\"" <<< "$lower"; then
  printf '%s\n' 'blocked: approval.toml is written by Brighton only (PIP-28)' >&2
  exit 2
fi

if grep -Eq -- "${path_field}(packs|pack-assembly)/[^\"]*(hidden|generator|known_ghosts|retired-values)\\.toml\"" <<< "$lower"; then
  printf '%s\n' 'LAY-07: a private file must not be written under the public packs folder (PIP-31)' >&2
  exit 2
fi

if grep -Eq -- "${path_field}(packs|pack-assembly)/[^\"]*meta\\.toml\"" <<< "$lower" \
  && grep -Eq -- 'status[[:space:]]*=[[:space:]]*\\?['\''"]published' <<< "$lower"; then
  printf '%s\n' 'blocked: only Brighton sets status to published, in the approval commit (PIP-1, PIP-28)' >&2
  exit 2
fi

exit 0
