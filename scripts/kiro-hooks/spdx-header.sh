#!/usr/bin/env bash
set -euo pipefail

file_path=${1:-}
event=$(cat)
if [[ -z "$file_path" ]] && command -v jq >/dev/null 2>&1; then
  file_path=$(printf '%s' "$event" | jq -r '.filePath // .path // ""' 2>/dev/null || true)
fi

if [[ -z "$file_path" ]]; then
  printf '%s\n' 'skipped: no created file path was supplied'
  exit 0
fi
if [[ ! -f "$file_path" ]]; then
  printf 'skipped: created file does not exist: %s\n' "$file_path"
  exit 0
fi

case "$file_path" in
  *.rs|*.ts) header='// SPDX-License-Identifier: MIT' ;;
  *.svelte) header='<!-- SPDX-License-Identifier: MIT -->' ;;
  *) exit 0 ;;
esac

if grep -Fqx "$header" "$file_path"; then
  exit 0
fi

tmp_file=$(mktemp)
trap 'rm -f "$tmp_file"' EXIT
{
  printf '%s\n' "$header"
  cat "$file_path"
} > "$tmp_file"
mv "$tmp_file" "$file_path"
trap - EXIT
