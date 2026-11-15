#!/usr/bin/env bash
# One-shot: when the current Kiro session (run-lifecycle) ends, stop only the
# exact demo-runner PIDs and start landing-steel-night (title-screen hero),
# then resume the demo queue from run-lifecycle onward.
# Usage: bash /tmp/kiro-landing-hero-handoff.sh <runner-pid> <watchdog-pid>
# Does not kill a running kiro-cli session. No broad pkill.
set -u
OLD=${1:?runner pid required}
WD=${2:?watchdog pid required}
REPO="/mnt/c/Users/Brighton/Downloads/Personal Projects/Counterspell"
P=/tmp/kiro-impl.progress
log(){ echo "$(date '+%F %H:%M') $*" >> "$P"; }

still_demo(){
  local pid=$1
  [ -d "/proc/$pid" ] || return 1
  tr '\0' ' ' < "/proc/$pid/cmdline" 2>/dev/null | grep -q 'kiro-implement-demo.sh'
}

if ! still_demo "$OLD"; then
  log "HANDOFF aborted: runner pid $OLD is not kiro-implement-demo.sh"
  exit 1
fi
if ! still_demo "$WD"; then
  log "HANDOFF aborted: watchdog pid $WD is not kiro-implement-demo.sh"
  exit 1
fi

log "HANDOFF armed: title-screen hero (landing-steel-night T21+) starts when the current Kiro session ends (runner $OLD, watchdog $WD)"

while still_demo "$OLD"; do
  if ! pgrep -P "$OLD" -f 'kiro-run.sh' >/dev/null; then
    kill -STOP "$OLD" 2>/dev/null || break
    if pgrep -P "$OLD" -f 'kiro-run.sh' >/dev/null; then
      kill -CONT "$OLD" 2>/dev/null || true
    else
      # Kill only children of the exact watchdog and runner, then those PIDs.
      for c in $(pgrep -P "$WD" 2>/dev/null); do kill "$c" 2>/dev/null || true; done
      kill "$WD" 2>/dev/null || true
      for c in $(pgrep -P "$OLD" 2>/dev/null); do kill "$c" 2>/dev/null || true; done
      kill -TERM "$OLD" 2>/dev/null || true
      kill -CONT "$OLD" 2>/dev/null || true
      break
    fi
  fi
  sleep 2
done

for i in $(seq 1 30); do tmux has-session -t kiro-impl 2>/dev/null || break; sleep 1; done
if tmux has-session -t kiro-impl 2>/dev/null; then
  if pgrep -f 'kiro-cli chat --v3' >/dev/null; then
    log "HANDOFF aborted: old session still open with Kiro running"
    exit 1
  fi
  tmux kill-session -t kiro-impl
fi
sleep 2
cd "$REPO" || exit 1
if [ -f .git/index.lock ] && ! pgrep -x git >/dev/null; then
  rm -f .git/index.lock
  log "HANDOFF removed stale index.lock"
fi
if [ ! -f scripts/kiro-implement-landing-hero-then-demo.sh ]; then
  log "HANDOFF aborted: scripts/kiro-implement-landing-hero-then-demo.sh missing"
  exit 1
fi
cp scripts/kiro-implement-landing-hero-then-demo.sh /tmp/kiro-implement-landing-hero-then-demo.sh
chmod +x /tmp/kiro-implement-landing-hero-then-demo.sh
tmux new -d -s kiro-impl -c "$REPO" 'bash /tmp/kiro-implement-landing-hero-then-demo.sh'
log "HANDOFF done: demo runner stopped between sessions; title-screen hero queue started in tmux kiro-impl"
