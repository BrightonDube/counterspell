#!/usr/bin/env bash
# Demo-first queue (15 Nov 2026, Brighton): finish the specs a judge sees first, looks first.
# Same mechanics as /tmp/kiro-implement-unblocked.sh (dev-test.env, preflight, kiro-run.sh,
# credit floor, leftover commits, continue past a stopped spec). Gates: judge-sandbox and
# language-drivers only while rootless Docker delegates cpuset and io.
# Usage: tmux new -d -s kiro-impl -c "$REPO" 'bash /tmp/kiro-implement-demo.sh' (a copy of scripts/kiro-implement-demo.sh)
set -u
REPO="/mnt/c/Users/Brighton/Downloads/Personal Projects/Counterspell"
cd "$REPO" || exit 1
. ~/.cargo/env
# Outside the repo: DATABASE_URL, VALKEY_URL, SANDBOX_DOCKER_SOCKET. Never print this file.
[ -f "$HOME/.config/counterspell/dev-test.env" ] && . "$HOME/.config/counterspell/dev-test.env"
export BREAK_ROOM_OK=1
P=/tmp/kiro-impl.progress
BRANCH=brighton/kiro-setup
FLOOR=250
WATCH_FLOOR=150
LIMIT=4000

usage(){ ~/.local/bin/kiro-cli chat --agent-engine v2 --no-interactive --trust-tools= "/usage" 2>/dev/null | grep -oE 'kiroween-2026: [0-9]+(\.[0-9]+)?' | awk '{print $2}'; }
log(){ echo "$(date '+%F %H:%M') $*" >> "$P"; }
WD=
stop(){ log "STOPPED $*"; [ -n "$WD" ] && kill "$WD" 2>/dev/null; exit 1; }

# Counts task lines with **T, **C, **PA, **G or **H ids (hints and the new specs
# do not all use **T). Stretch is only a [stretch] / [STRETCH] marker on the task line.
counts(){
  local f=.kiro/specs/$1/tasks.md
  echo "$(grep -cE '^\s*- \[x\] \*\*[A-Z]' "$f" || true) $(grep -cE '^\s*- \[( |x)\] \*\*[A-Z]' "$f" || true) $(grep -E '^\s*- \[( |x)\] \*\*[A-Z]' "$f" | grep -ciE '\[stretch\]' || true) $(grep -cE '^\s*- \[ \] \*\*[A-Z].*BLOCKED' "$f" || true)";
}

suite(){
  { cargo test --workspace && (cd web && npm run check && npm run build && npm run test --if-present); } > /tmp/kiro-impl-tests.log 2>&1;
}

credit_ok(){
  local why=$1 est=${2:-0}
  U=$(usage); [ -n "$U" ] || stop "could not read credits from /usage"
  REM=$(awk -v u="$U" -v l=$LIMIT 'BEGIN{printf "%.0f", l-u}')
  [ "$REM" -ge "$FLOOR" ] || stop "remaining credits $REM < $FLOOR $why"
  [ $(( REM - est )) -ge "$FLOOR" ] || stop "remaining $REM minus estimate $est would go below $FLOOR $why"
}

if [ -f scripts/kiro-pending-spec-amendments.sh ]; then
  bash scripts/kiro-pending-spec-amendments.sh || stop "pending spec amendment failed"
fi

echo "----" >> "$P"
[ "$(git branch --show-current)" = "$BRANCH" ] || stop "not on $BRANCH"
log "START demo-first queue; preflight test suite"
suite || stop "preflight: tests already failing before any spec (see /tmp/kiro-impl-tests.log)"
log "preflight passed"

( while sleep 1200; do u=$(usage); [ -n "$u" ] || continue
    if awk -v u="$u" -v l=$LIMIT -v f=$WATCH_FLOOR 'BEGIN{exit !(l-u < f)}'; then
      echo "$(date '+%F %H:%M') WATCHDOG remaining below $WATCH_FLOOR (used $u); killing Kiro" >> "$P"
      if pid=$(pgrep -n -f 'kiro-cli chat --v3'); then kill "$pid" 2>/dev/null; fi
    fi
  done ) &
WD=$!

run_spec(){
  local NUM=$1 SLUG=$2 MODEL=$3 EST=$4
  local PF=docs/kiro-prompts/$NUM-$SLUG-implement-cli.md
  [ -f "$PF" ] || { log "SKIPPED $SLUG: missing prompt $PF"; return 0; }
  [ "$(git branch --show-current)" = "$BRANCH" ] || stop "branch changed away from $BRANCH"
  read -r D T S B <<<"$(counts "$SLUG")"
  if [ "$D" -ge $(( T - S - B )) ]; then
    if [ "${B:-0}" -gt 0 ]; then log "$SLUG already done ($D/$T, $B blocked)"; else log "$SLUG already done ($D/$T)"; fi
    return 0
  fi
  credit_ok "before $SLUG" "$EST"
  log "BEGIN $SLUG model=$MODEL est=$EST remaining=$REM"
  for ATT in 1 2 3 4 5; do
    read -r D0 T S B <<<"$(counts "$SLUG")"
    [ "$D0" -ge $(( T - S - B )) ] && break
    credit_ok "during $SLUG" 0
    CONT=/tmp/kiro-continue-$SLUG-a$ATT.md
    {
      echo "This is a continuation (attempt $ATT). Resume at the first unchecked, non-stretch, non-BLOCKED, non-HUMAN task in .kiro/specs/$SLUG/tasks.md. Do not redo tasks already marked [x]. Skip [BLOCKED] and [stretch] and [HUMAN] tasks (leave a note in the blockers file if you have something new to add) and keep going. If a PreToolUse hook blocks a write, read its message; never bypass the hook with a shell write. A hook block on one task is not a reason to end the run. If a task needs something only Brighton can do, record it and move to the next task that does not. Visual quality is the top priority for the demo: on every UI task apply .kiro/skills/impeccable and use the approved art in web/static/art/ (brighton-decisions.md, 15 Nov 2026 14:00)."
      echo
      cat "$PF"
    } > "$CONT"
    bash scripts/kiro-run.sh "$NUM-$SLUG-implement-a$ATT" "$MODEL" default "$CONT"
    read -r D1 T S B <<<"$(counts "$SLUG")"
    log "$SLUG attempt $ATT: tasks $D0 -> $D1 of $T ($(grep -o 'rc=[0-9]*' /tmp/kiro-run.status), $(grep -o 'credits=[0-9.-]*' /tmp/kiro-run.status))"
    [ "$D1" -gt "$D0" ] || break
  done
  RC=$(grep -o 'rc=[0-9]*' /tmp/kiro-run.status | cut -d= -f2)
  CR=$(grep -o 'credits=[0-9.-]*' /tmp/kiro-run.status | cut -d= -f2)
  TOT=$(grep -o 'total=[0-9.]*' /tmp/kiro-run.status | cut -d= -f2)
  read -r D T S B <<<"$(counts "$SLUG")"
  if suite; then TESTS=pass; else TESTS=FAIL; fi
  git add -A && git commit -qm "kiro: $SLUG implement run log and leftovers ($MODEL)" || true
  log "$SLUG rc=$RC credits=$CR total=$TOT tasks=$D/$T stretch=$S blocked=$B tests=$TESTS"
  if [ "$TESTS" != pass ]; then
    log "CONTINUE after $SLUG: tests failing (see /tmp/kiro-impl-tests.log)"
    return 0
  fi
  if [ $(( D * 2 )) -lt $(( T - S - B )) ]; then
    log "CONTINUE after $SLUG: $D of $(( T - S - B )) core tasks checked (under half)"
    return 0
  fi
}

run_brief(){
  local LABEL=$1 MODEL=$2 PF=$3 EST=$4
  [ -f "$PF" ] || { log "SKIPPED $LABEL: missing prompt $PF"; return 0; }
  [ "$(git branch --show-current)" = "$BRANCH" ] || stop "branch changed away from $BRANCH"
  credit_ok "before $LABEL" "$EST"
  log "BEGIN $LABEL model=$MODEL est=$EST remaining=$REM"
  CONT=/tmp/kiro-continue-$LABEL.md
  {
    echo "This brief is APPROVED by Brighton (15 Nov 2026). Implement it now, non-interactively. Follow AGENTS.md and every file in .kiro/steering/ (especially tdd-and-quality.md, security.md and brighton-decisions.md). Strict TDD. Do not push, do not change branches, never spend money, never provision a server, never read or write .env files. The private repo BrightonDube/counterspell-packs-private exists (see brighton-decisions.md). If a step needs a purchase, log it and continue with work that does not. Use the limited importer database role already decided on 15 Nov 2026. If a PreToolUse hook blocks a write, read its message; never bypass the hook with a shell write."
    echo
    cat "$PF"
  } > "$CONT"
  bash scripts/kiro-run.sh "$LABEL" "$MODEL" default "$CONT"
  RC=$(grep -o 'rc=[0-9]*' /tmp/kiro-run.status | cut -d= -f2)
  CR=$(grep -o 'credits=[0-9.-]*' /tmp/kiro-run.status | cut -d= -f2)
  TOT=$(grep -o 'total=[0-9.]*' /tmp/kiro-run.status | cut -d= -f2)
  if suite; then TESTS=pass; else TESTS=FAIL; fi
  git add -A && git commit -qm "kiro: $LABEL implement run log and leftovers ($MODEL)" || true
  log "$LABEL rc=$RC credits=$CR total=$TOT tests=$TESTS"
  if [ "$TESTS" != pass ]; then
    log "CONTINUE after $LABEL: tests failing (see /tmp/kiro-impl-tests.log)"
  fi
}

half_done(){ read -r d t s b <<<"$(counts "$1")"; [ $(( d * 2 )) -ge $(( t - s - b )) ]; }
deleg_ok(){ local f=/sys/fs/cgroup/user.slice/user-1000.slice/user@1000.service/cgroup.subtree_control
  grep -qw cpuset "$f" 2>/dev/null && grep -qw io "$f" 2>/dev/null && ! docker --context rootless info 2>&1 | grep -qiE 'no (cpuset|io)'; }
gate(){ case $1 in
    judge-sandbox) deleg_ok || { echo "rootless Docker lacks cpuset/io delegation"; return 1; }
      grep -q "Review decisions" .kiro/specs/judge-sandbox/design.md || { echo "T00 boundary review not signed off"; return 1; } ;;
    language-drivers) deleg_ok || { echo "rootless Docker lacks cpuset/io delegation"; return 1; }
      half_done judge-sandbox || { echo "needs judge-sandbox at least half implemented"; return 1; } ;;
    counterspell-mcp-power) half_done judge-sandbox || { echo "needs judge-sandbox at least half implemented"; return 1; } ;;
  esac; }
gated(){ local why; if ! why=$(gate "$2"); then log "SKIPPED $2: $why"; return 0; fi; run_spec "$@"; }

gated 34 problem-catalogue claude-haiku-5.5 25
gated 31 landing-steel-night claude-haiku-5.5 12
gated 21 grimoire-packs claude-haiku-5.5 10
gated 25 auth claude-haiku-5.5 15
gated 22 run-lifecycle claude-haiku-5.5 18
gated 26 ide-workspace claude-haiku-5.5 15
gated 27 efficiency-familiar claude-haiku-5.5 19
gated 29 dashboard-fsrs claude-haiku-5.5 10
gated 23 judge-sandbox claude-haiku-5.5 30
gated 24 language-drivers claude-haiku-5.5 15
gated 35 deploy-ops claude-haiku-5.5 20
gated 36 pack-authoring-pipeline claude-haiku-5.5 20
run_brief 33-starter-pack-hardening-v2 claude-haiku-5.5 docs/kiro-prompts/33-starter-pack-hardening-v2.md 15
gated 28 hints claude-haiku-5.5 15
gated 32 break-room claude-haiku-5.5 10
gated 30 counterspell-mcp-power claude-haiku-5.5 10
log "ALL_DONE"
kill "$WD" 2>/dev/null
