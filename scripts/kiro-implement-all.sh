#!/usr/bin/env bash
# Implements the specs strictly one at a time, in the order of docs/kiro-prompts/19-implementation-order.md.
# Usage: tmux new -d -s kiro-impl 'bash scripts/kiro-implement-all.sh'      (all, in order)
#        bash scripts/kiro-implement-all.sh judge-sandbox language-drivers  (only the named slugs, still in order)
# Progress: /tmp/kiro-impl.progress (one line per spec; ends with ALL_DONE or STOPPED <reason>).
set -u
cd "$(dirname "$0")/.."
. ~/.cargo/env
P=/tmp/kiro-impl.progress
BRANCH=brighton/kiro-setup
FLOOR=250          # never start a spec if remaining credits are below this
WATCH_FLOOR=150    # kill a running spec if remaining credits drop below this
LIMIT=4000
# num slug model estimated-credits
SPECS=(
  "20 ghost-engine gpt-5.6-luna 95"
  "21 grimoire-packs gpt-5.6-luna 75"
  "22 run-lifecycle gpt-5.6-luna 140"
  "23 judge-sandbox claude-sonnet-5.5 365"
  "24 language-drivers gpt-5.6-luna 55"
  "25 auth gpt-5.6-luna 140"
  "26 ide-workspace gpt-5.6-luna 100"
  "27 efficiency-familiar gpt-5.6-luna 150"
  "28 hints gpt-5.6-luna 75"
  "29 dashboard-fsrs gpt-5.6-luna 70"
  "30 counterspell-mcp-power gpt-5.6-luna 80"
  "31 landing-steel-night gpt-5.6-luna 60"
  "32 break-room gpt-5.6-luna 55"
)
usage(){ ~/.local/bin/kiro-cli chat --no-interactive --model claude-haiku-5.5 --trust-tools= "/usage" 2>/dev/null | grep -o 'kiroween-2026: [0-9.]*' | awk '{print $2}'; }
log(){ echo "$(date '+%F %H:%M') $*" >> "$P"; }
WD=
stop(){ log "STOPPED $*"; [ -n "$WD" ] && kill "$WD" 2>/dev/null; exit 1; }
counts(){ # slug -> "done total stretch"
  local f=.kiro/specs/$1/tasks.md
  echo "$(grep -cE '^\s*- \[x\]' "$f") $(grep -cE '^\s*- \[( |x)\]' "$f") $(grep -E '^\s*- \[( |x)\]' "$f" | grep -ci stretch)"; }
half_done(){ read -r d t s <<<"$(counts "$1")"; [ $(( d * 2 )) -ge $(( t - s )) ]; }
gate(){ # slug -> 0 if allowed to start; prints the reason otherwise
  case $1 in
    judge-sandbox) grep -q "Review decisions" .kiro/specs/judge-sandbox/design.md || grep -qE '^\s*- \[x\] \*\*T00' .kiro/specs/judge-sandbox/tasks.md || { echo "T00 boundary review not signed off (no 'Review decisions' note in design.md)"; return 1; } ;;
    language-drivers|counterspell-mcp-power) half_done judge-sandbox || { echo "needs judge-sandbox at least half implemented"; return 1; } ;;
    break-room) [ "${BREAK_ROOM_OK:-0}" = 1 ] || { echo "T00 needs the freeze-safe core verified on the VPS (set BREAK_ROOM_OK=1 once it is)"; return 1; } ;;
  esac; }
suite(){ # full test suite on the branch; output in /tmp/kiro-impl-tests.log
  { cargo test --workspace && (cd web && npm run check && npm run build && npm run test --if-present); } > /tmp/kiro-impl-tests.log 2>&1; }

: > "$P"
[ "$(git branch --show-current)" = "$BRANCH" ] || stop "not on $BRANCH"
log "START preflight test suite"
suite || stop "preflight: tests already failing before any spec (see /tmp/kiro-impl-tests.log)"
# Watchdog: a spec that overruns its estimate must not eat the last credits.
( while sleep 1200; do u=$(usage); [ -n "$u" ] || continue
    if awk -v u="$u" -v l=$LIMIT -v f=$WATCH_FLOOR 'BEGIN{exit !(l-u < f)}'; then
      echo "$(date '+%F %H:%M') WATCHDOG remaining below $WATCH_FLOOR (used $u); killing Kiro" >> "$P"; pkill -f 'kiro-cli chat --v3'; fi; done ) &
WD=$!
for row in "${SPECS[@]}"; do
  read -r NUM SLUG MODEL EST <<<"$row"
  if [ $# -gt 0 ] && ! printf '%s\n' "$@" | grep -qx "$SLUG"; then continue; fi
  read -r D T S <<<"$(counts "$SLUG")"
  if [ "$D" -ge $(( T - S )) ]; then log "$SLUG already done ($D/$T)"; continue; fi
  if ! why=$(gate "$SLUG"); then log "SKIPPED $SLUG: $why"; continue; fi
  [ "$(git branch --show-current)" = "$BRANCH" ] || stop "branch changed away from $BRANCH"
  U=$(usage); [ -n "$U" ] || stop "could not read credits from /usage"
  REM=$(awk -v u="$U" -v l=$LIMIT 'BEGIN{printf "%.0f", l-u}')
  [ "$REM" -ge "$FLOOR" ] || stop "remaining credits $REM < $FLOOR before $SLUG"
  [ $(( REM - EST )) -ge "$FLOOR" ] || stop "remaining $REM minus $SLUG estimate $EST would go below $FLOOR"
  log "BEGIN $SLUG model=$MODEL est=$EST remaining=$REM"
  bash scripts/kiro-run.sh "$NUM-$SLUG-implement" "$MODEL" default "docs/kiro-prompts/$NUM-$SLUG-implement-cli.md"
  RC=$(grep -o 'rc=[0-9]*' /tmp/kiro-run.status | cut -d= -f2)
  CR=$(grep -o 'credits=[0-9.-]*' /tmp/kiro-run.status | cut -d= -f2)
  TOT=$(grep -o 'total=[0-9.]*' /tmp/kiro-run.status | cut -d= -f2)
  read -r D T S <<<"$(counts "$SLUG")"
  if suite; then TESTS=pass; else TESTS=FAIL; fi
  git add -A && git commit -qm "kiro: $SLUG implement run log and leftovers ($MODEL)" || true
  log "$SLUG rc=$RC credits=$CR total=$TOT tasks=$D/$T stretch=$S tests=$TESTS"
  [ "$TESTS" = pass ] || stop "tests failing after $SLUG (see /tmp/kiro-impl-tests.log)"
  [ $(( D * 2 )) -ge $(( T - S )) ] || stop "$SLUG ended with $D of $(( T - S )) core tasks checked (under half)"
done
log "ALL_DONE"
kill "$WD" 2>/dev/null
