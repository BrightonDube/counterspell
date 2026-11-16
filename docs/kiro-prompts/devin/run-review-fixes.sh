#!/usr/bin/env bash
# Launcher for the Devin review-fix Kiro resume run (14 Nov 2026). Started inside tmux by the driver.
set -u
cd "/mnt/c/Users/Brighton/Downloads/Personal Projects/Counterspell" || exit 1
LOG="docs/kiro-log/$(date +%Y-%m-%dT%H%M)-devin-testerows-fix.log"
~/.local/bin/kiro-cli chat --v3 --mode default --no-interactive --trust-workspace -a \
  --model claude-haiku-5.5 \
  "$(cat docs/kiro-prompts/devin/2026-11-14-followup-testerows-internal-error.md)" \
  > "$LOG" 2>&1
echo "exit=$? log=$LOG" > "docs/kiro-log/$(date +%Y-%m-%dT%H%M)-devin-testerows-fix.exit"
