#!/usr/bin/env bash
set -euo pipefail

event=$(cat)
tmp_file=$(mktemp)
trap 'rm -f "$tmp_file"' EXIT

if [[ -f docs/plan.md ]]; then
  printf '%s\n' 'today'
  sed -n '1,20p' docs/plan.md >> "$tmp_file"
else
  printf '%s\n' 'no plan.md'
fi

if command -v git >/dev/null 2>&1 && git rev-parse --show-toplevel >/dev/null 2>&1; then
  {
    printf '%s\n' 'recent commits:'
    git log --oneline -3
  } >> "$tmp_file"
else
  printf '%s\n' 'skipped: Git history is unavailable'
fi

printf '%s\n' 'open bugfix specs:' >> "$tmp_file"
found=0
shopt -s nullglob
for task_file in .kiro/specs/bugfix-*/tasks.md; do
  found=1
  printf '%s\n' "- $task_file" >> "$tmp_file"
  grep -E '^[-*] \[ \]' "$task_file" | head -5 >> "$tmp_file" || true
done
shopt -u nullglob
if [[ "$found" -eq 0 ]]; then
  printf '%s\n' '- none found' >> "$tmp_file"
fi

sed -n '1,40p' "$tmp_file"
