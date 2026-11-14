#!/usr/bin/env bash
# Wait for the already-running kiro-impl session to finish on its own, then start
# judge-sandbox and language-drivers. Does not kill tmux, kiro-cli, or any runner.
# One Kiro session at a time.
set -u
cd "$(dirname "$0")/.."
Q=/tmp/kiro-queue.progress
log(){ echo "$(date '+%F %H:%M') $*" >> "$Q"; }
log "waiting for current kiro-impl to finish (no kill) before requeueing judge-sandbox then language-drivers"
while tmux has-session -t kiro-impl 2>/dev/null; do
  sleep 15
done
for _ in 1 2 3 4 5 6 7 8; do
  if pgrep -f 'kiro-cli chat --v3' >/dev/null; then sleep 10; else break; fi
done
if pgrep -f 'kiro-cli chat --v3' >/dev/null; then
  log "STOPPED a Kiro session is still running after kiro-impl ended; not starting another"
  exit 1
fi
sleep 5
if tmux has-session -t kiro-impl 2>/dev/null; then
  log "STOPPED kiro-impl already exists; not starting a second session"
  exit 1
fi
if [ "$(git branch --show-current)" != brighton/kiro-setup ]; then
  log "STOPPED not on brighton/kiro-setup"
  exit 1
fi
tmux new -d -s kiro-impl 'bash scripts/kiro-implement-all.sh judge-sandbox language-drivers >> /mnt/c/Users/Brighton/kiro-impl.log 2>&1'
log "started kiro-impl: judge-sandbox, then language-drivers"
