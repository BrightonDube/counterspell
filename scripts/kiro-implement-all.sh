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
  "20 ghost-engine claude-haiku-5.5 12"
  "21 grimoire-packs claude-haiku-5.5 10"
  "22 run-lifecycle claude-haiku-5.5 18"
  "23 judge-sandbox claude-haiku-5.5 46"
  "24 language-drivers claude-haiku-5.5 10"
  "25 auth claude-haiku-5.5 18"
  "26 ide-workspace claude-haiku-5.5 12"
  "27 efficiency-familiar claude-haiku-5.5 19"
  "28 hints claude-haiku-5.5 10"
  "29 dashboard-fsrs claude-haiku-5.5 10"
  "30 counterspell-mcp-power claude-haiku-5.5 10"
  "31 landing-steel-night claude-haiku-5.5 10"
  "32 break-room claude-haiku-5.5 10"
)
usage(){ ~/.local/bin/kiro-cli chat --agent-engine v2 --no-interactive --trust-tools= "/usage" 2>/dev/null | grep -oE 'kiroween-2026: [0-9]+(\.[0-9]+)?' | awk '{print $2}'; }  # v2 runs /usage locally; v3 (chat.agentEngine=v3 since 3.3.0) sends it to the model as a prompt
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
  # Re-run the same spec while each attempt makes progress (Kiro sessions can be cut off mid-spec).
  for ATT in 1 2 3 4 5; do
    read -r D0 T S <<<"$(counts "$SLUG")"
    [ "$D0" -ge $(( T - S )) ] && break
    U=$(usage); [ -n "$U" ] || stop "could not read credits from /usage"
    REM=$(awk -v u="$U" -v l=$LIMIT 'BEGIN{printf "%.0f", l-u}')
    [ "$REM" -ge "$FLOOR" ] || stop "remaining credits $REM < $FLOOR during $SLUG"
    bash scripts/kiro-run.sh "$NUM-$SLUG-implement-a$ATT" "$MODEL" default "docs/kiro-prompts/$NUM-$SLUG-implement-cli.md"
    read -r D1 T S <<<"$(counts "$SLUG")"
    log "$SLUG attempt $ATT: tasks $D0 -> $D1 of $T ($(grep -o 'rc=[0-9]*' /tmp/kiro-run.status), $(grep -o 'credits=[0-9.-]*' /tmp/kiro-run.status))"
    [ "$D1" -gt "$D0" ] || break
  done
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
