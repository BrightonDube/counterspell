#!/usr/bin/env bash
# Waits for the running kiro-impl runner to finish its current Kiro attempt, stops it before it starts
# another one, then starts scripts/kiro-gap-specs.sh in a fresh tmux session named kiro-impl.
# Guarantees one Kiro session at a time. Run: tmux new -d -s kiro-queue 'bash scripts/kiro-after-current.sh'
set -u
cd "$(dirname "$0")/.."
P=/tmp/kiro-impl.progress
Q=/tmp/kiro-queue.progress
log(){ echo "$(date '+%F %H:%M') $*" >> "$Q"; }
START=$(wc -l < "$P")
log "waiting for the current kiro-impl attempt to end (progress line $START)"
while tmux has-session -t kiro-impl 2>/dev/null; do
  if tail -n +$((START+1)) "$P" | grep -qE 'attempt [0-9]+:| rc=|STOPPED|ALL_DONE|BEGIN '; then
    tmux kill-session -t kiro-impl 2>/dev/null
    sleep 2
    pkill -f 'kiro-implement-all.sh' 2>/dev/null
    pkill -f 'kiro-cli chat --v3' 2>/dev/null
    log "stopped kiro-impl after: $(tail -1 "$P")"
    break
  fi
  sleep 3
done
sleep 5
if pgrep -f 'kiro-cli chat --v3' >/dev/null; then log "STOPPED a Kiro session is still running; not starting another"; exit 1; fi
tmux new -d -s kiro-impl 'bash scripts/kiro-gap-specs.sh'
log "started kiro-impl: gap specs, then judge-sandbox, language-drivers, auth, ide-workspace, efficiency-familiar, hints, dashboard-fsrs, counterspell-mcp-power, landing-steel-night, break-room"
