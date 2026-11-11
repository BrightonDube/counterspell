#!/usr/bin/env bash
# Usage: scripts/kiro-run.sh <phase-label> <model> <mode:spec|default> <prompt-file>
set -o pipefail
cd "$(dirname "$0")/.."
. ~/.cargo/env
LABEL=$1; MODEL=$2; MODE=$3; PF=$4
usage(){ ~/.local/bin/kiro-cli chat --no-interactive --model claude-haiku-5.5 --trust-tools= "/usage" 2>/dev/null | grep -o 'kiroween-2026: [0-9.]*' | awk '{print $2}'; }
U0=$(usage); TS=$(date +%Y-%m-%dT%H%M); LOG=docs/kiro-log/${TS}-${LABEL}.log; START=$(date '+%H:%M')
~/.local/bin/kiro-cli chat --v3 --mode "$MODE" --no-interactive --trust-workspace -a --model "$MODEL" "$(cat "$PF")" > "$LOG" 2>&1
RC=$?
U1=$(usage)
DELTA=$(awk -v a="$U0" -v b="$U1" 'BEGIN{printf "%.2f", b-a}')
printf '\n### %s %s-%s SAST: %s (Kiro CLI, %s, mode %s)\n- Prompt: `%s`. Transcript: `%s`. Exit %s.\n- Credits used (from /usage): %s (hackathon credits used now %s/4000).\n' "$(date +%F)" "$START" "$(date +%H:%M)" "$LABEL" "$MODEL" "$MODE" "$PF" "$LOG" "$RC" "$DELTA" "$U1" >> docs/KIRO_USAGE.md
echo "DONE $LABEL rc=$RC credits=$DELTA total=$U1" > /tmp/kiro-run.status