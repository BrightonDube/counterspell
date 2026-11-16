#!/usr/bin/env bash
# One-shot: waits for the current kiro-impl tmux runner to exit on its own (demo queue ALL_DONE
# or STOPPED), then starts scripts/kiro-sandbox-then-drivers.sh as kiro-impl. Never kills anything.
cd "$(dirname "$0")/.."
P=/tmp/kiro-impl.progress
echo "$(date '+%F %H:%M') HANDOFF armed: judge-sandbox then language-drivers after the demo queue" >> "$P"
while tmux has-session -t kiro-impl 2>/dev/null; do sleep 20; done
sleep 10
while pgrep -f 'kiro-cli chat --v3' >/dev/null; do sleep 20; done
tmux new -d -s kiro-impl -c "$PWD" 'bash scripts/kiro-sandbox-then-drivers.sh'
echo "$(date '+%F %H:%M') HANDOFF started judge-sandbox then language-drivers" >> "$P"
