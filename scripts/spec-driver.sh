#!/usr/bin/env bash
# Start, stop and watch the spec-driver Kiro agent.
#   scripts/spec-driver.sh start "<command>" [minutes]   e.g. start "drive judge-sandbox" 60
#   scripts/spec-driver.sh attach | status | stop
#   scripts/spec-driver.sh context | turn-end             (used by the agent's own hooks)
# Interactive alternative: kiro-cli chat --agent spec-driver   (Ctrl+C to stop)
set -u
cd "$(dirname "$0")/.." || exit 1
S=spec-driver
PIDF=/tmp/spec-driver.pid
LOG=/tmp/spec-driver.log
case "${1:-}" in
  start)
    CMD=${2:?say what to drive, e.g. "drive judge-sandbox"}; MIN=${3:-60}
    tmux has-session -t "$S" 2>/dev/null && { echo "already running (scripts/spec-driver.sh attach)"; exit 1; }
    tmux has-session -t kiro-impl 2>/dev/null && { echo "the kiro-impl runner is active; stop it first so only one Kiro session edits the repo"; exit 1; }
    . ~/.cargo/env 2>/dev/null
    tmux new -d -s "$S" -c "$PWD" "bash -c 'echo \$\$ > $PIDF; [ -f ~/.config/counterspell/dev-test.env ] && . ~/.config/counterspell/dev-test.env; exec timeout --signal=INT ${MIN}m ~/.local/bin/kiro-cli chat --agent spec-driver --no-interactive --trust-all-tools \"$CMD\" 2>&1 | tee -a $LOG'"
    echo "$(date '+%F %H:%M') START spec-driver: $CMD (${MIN} min)" >> docs/kiro-log/spec-driver-runs.md
    echo "started: $CMD for up to ${MIN} min. Watch: scripts/spec-driver.sh attach (detach with Ctrl+B then D)";;
  attach) tmux attach -t "$S";;
  status)
    tmux has-session -t "$S" 2>/dev/null && echo "running" || echo "not running"
    git log --oneline -5; tail -5 "$LOG" 2>/dev/null;;
  stop)
    if [ -f "$PIDF" ] && kill -0 "$(cat $PIDF)" 2>/dev/null; then
      kill -INT "$(cat $PIDF)"; sleep 8; kill -0 "$(cat $PIDF)" 2>/dev/null && kill -TERM "$(cat $PIDF)"
    fi
    tmux kill-session -t "$S" 2>/dev/null; rm -f "$PIDF"
    echo "$(date '+%F %H:%M') STOP spec-driver (last commit $(git log -1 --format=%h))" >> docs/kiro-log/spec-driver-runs.md
    echo "stopped; last commit is the safe point";;
  context)
    echo "branch: $(git branch --show-current)"; git status --short | head -10
    for f in .kiro/specs/*/tasks.md; do d=$(basename "$(dirname "$f")"); x=$(grep -cE '^\s*- \[x\] \*\*' "$f"); t=$(grep -cE '^\s*- \[( |x)\] \*\*' "$f"); echo "$d $x/$t"; done;;
  turn-end)
    echo "$(date '+%F %H:%M') turn end, last commit $(git log -1 --format='%h %s')" >> docs/kiro-log/spec-driver-runs.md;;
  *) sed -n 2,7p "$0";;
esac
