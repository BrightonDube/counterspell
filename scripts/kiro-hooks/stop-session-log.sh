#!/usr/bin/env bash
set -euo pipefail

event=$(cat)
if ! command -v git >/dev/null 2>&1 || ! git rev-parse --show-toplevel >/dev/null 2>&1; then
  printf '%s\n' 'skipped: this directory is not a Git worktree'
  exit 0
fi

if git diff --quiet && git diff --cached --quiet; then
  printf '%s\n' 'session stop log: skipped because there are no tracked changes'
  exit 0
fi

mkdir -p docs/kiro-log
model='not reported'
if command -v jq >/dev/null 2>&1; then
  model=$(printf '%s' "$event" | jq -r '.model // .modelId // "not reported"' 2>/dev/null || true)
fi
[[ -n "$model" ]] || model='not reported'
{
  printf -- '- %s | model: %s | last commit: ' "$(date -u +%Y-%m-%dT%H:%M:%SZ)" "$model"
  git log -1 --format='%h %s' 2>/dev/null || printf '%s' 'unknown'
  printf '%s\n' ' | diff stat:'
  git diff --stat 2>/dev/null || true
  git diff --cached --stat 2>/dev/null || true
} >> docs/kiro-log/sessions.md
