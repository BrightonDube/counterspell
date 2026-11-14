#!/usr/bin/env bash
# Runs pending spec-only amendments that must land before implementation.
# Safe on every implement-all start: no-ops when the amendment is already applied
# or the prompt is missing. One Kiro session at a time.
set -u
cd "$(dirname "$0")/.."
Q=/tmp/kiro-queue.progress
P=/tmp/kiro-impl.progress
LIMIT=4000
FLOOR=200
logq(){ echo "$(date '+%F %H:%M') $*" >> "$Q"; }
logp(){ echo "$(date '+%F %H:%M') $*" >> "$P"; }
usage(){ ~/.local/bin/kiro-cli chat --agent-engine v2 --no-interactive --trust-tools= "/usage" 2>/dev/null | grep -oE 'kiroween-2026: [0-9]+(\.[0-9]+)?' | awk '{print $2}'; }

f=docs/kiro-prompts/38-private-pack-split.md
specdir=.kiro/specs/problem-catalogue
if [ ! -f "$f" ] || [ ! -f "$specdir/tasks.md" ]; then
  exit 0
fi
if grep -q 'counterspell-packs-private' "$specdir/design.md" "$specdir/requirements.md" 2>/dev/null; then
  logq "38-private-pack-split already reflected in problem-catalogue, skipped"
  exit 0
fi
if pgrep -f 'kiro-cli chat --v3' >/dev/null; then
  logq "STOPPED 38-private-pack-split: a Kiro session is still running"
  logp "STOPPED 38-private-pack-split: a Kiro session is still running"
  exit 1
fi
U=$(usage); [ -n "$U" ] || { logq "STOPPED could not read credits before 38-private-pack-split"; exit 1; }
REM=$(awk -v u="$U" -v l=$LIMIT 'BEGIN{printf "%.0f", l-u}')
if [ "$REM" -lt "$FLOOR" ]; then
  logq "STOPPED remaining credits $REM < $FLOOR before 38-private-pack-split"
  logp "STOPPED remaining credits $REM < $FLOOR before 38-private-pack-split"
  exit 1
fi
logq "BEGIN 38-private-pack-split (claude-haiku-5.5) remaining=$REM"
logp "BEGIN 38-private-pack-split remaining=$REM"
bash scripts/kiro-run.sh "38-private-pack-split" "claude-haiku-5.5" spec "$f"
RC=$?
git add .kiro/specs/problem-catalogue docs/kiro-prompts/38-private-pack-split.md docs/KIRO_USAGE.md docs/kiro-log/*-38-private-pack-split.log 2>/dev/null
git commit -qm "kiro: amend problem-catalogue for the private-pack split (Kiro CLI spec mode, claude-haiku-5.5)" || true
logq "38-private-pack-split $(cat /tmp/kiro-run.status 2>/dev/null)"
logp "38-private-pack-split $(cat /tmp/kiro-run.status 2>/dev/null)"
[ "$RC" -eq 0 ] || exit "$RC"
exit 0
