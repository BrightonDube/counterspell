#!/usr/bin/env bash
set -euo pipefail

event=$(cat)
tmp_file=$(mktemp)
trap 'rm -f "$tmp_file"' EXIT

printf '%s\n' 'current spec task list:' >> "$tmp_file"
shopt -s nullglob
found=0
for task_file in .kiro/specs/*/tasks.md; do
  found=1
  printf '%s\n' "[$task_file]" >> "$tmp_file"
  grep -E '^[-*] \[ \]' "$task_file" | head -10 >> "$tmp_file" || true
done
shopt -u nullglob
if [[ "$found" -eq 0 ]]; then
  printf '%s\n' '- no spec task files found' >> "$tmp_file"
fi

printf '%s\n' 'git status:' >> "$tmp_file"
if command -v git >/dev/null 2>&1 && git rev-parse --show-toplevel >/dev/null 2>&1; then
  git status --short >> "$tmp_file"
else
  printf '%s\n' '- skipped: Git history is unavailable' >> "$tmp_file"
fi

sed -n '1,40p' "$tmp_file"
