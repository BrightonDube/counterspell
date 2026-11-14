#!/usr/bin/env bash
# Writes the specs for the gaps found on 14 Nov 2026 (spec mode only, no implementation),
# then resumes implementation with judge-sandbox, auth and ide-workspace first.
# Run inside tmux session kiro-impl (scripts/kiro-after-current.sh starts it). One Kiro session at a time.
set -u
cd "$(dirname "$0")/.."
. ~/.cargo/env
Q=/tmp/kiro-queue.progress
LIMIT=4000
SPEC_FLOOR=200   # do not start a spec-writing session below this many remaining credits
log(){ echo "$(date '+%F %H:%M') $*" >> "$Q"; }
usage(){ ~/.local/bin/kiro-cli chat --agent-engine v2 --no-interactive --trust-tools= "/usage" 2>/dev/null | grep -oE 'kiroween-2026: [0-9]+(\.[0-9]+)?' | awk '{print $2}'; }
[ "$(git branch --show-current)" = brighton/kiro-setup ] || { log "STOPPED not on brighton/kiro-setup"; exit 1; }
spec(){ # num slug model
  local f=docs/kiro-prompts/$1-$2.md out=docs/kiro-prompts/$1-$2-cli.md
  if [ -f ".kiro/specs/$2/tasks.md" ]; then log "$2 spec already exists, skipped"; return 0; fi
  local U REM; U=$(usage); [ -n "$U" ] || { log "STOPPED could not read credits"; exit 1; }
  REM=$(awk -v u="$U" -v l=$LIMIT 'BEGIN{printf "%.0f", l-u}')
  [ "$REM" -ge "$SPEC_FLOOR" ] || { log "STOPPED remaining credits $REM < $SPEC_FLOOR before $2 spec"; exit 1; }
  { sed -n '/^```$/,/^```$/p' "$f" | sed '1d;$d' | sed 's/#\[\[file:\([^]]*\)\]\]/\1/g'
    printf '\nThis run is non-interactive (Kiro CLI). Read the referenced docs and .kiro/steering/ yourself (brighton-decisions.md first). Write the complete spec (requirements.md, design.md, tasks.md) under .kiro/specs/%s/ in one go, then stop. Use the same structure and conventions as the existing specs in .kiro/specs/ (numbered EARS requirements, design with diagrams, correctness properties and an "Open questions" section with the assumed choice for each, tasks.md with waves, T00-style gates where needed, [P]/[S]/[SUPERVISED]/[HUMAN] markers, 30-minute tasks, per-task files, requirements, dependencies, verification and a Rust lesson for Rust tasks). This is the system-design stage: do NOT implement any task, and do not create or change any file outside .kiro/specs/%s/. Keep consistent with .kiro/specs/repo-skeleton/ (layout, crate names) and every other spec; where you depend on another spec, name its task ids.\n' "$2" "$2"; } > "$out"
  log "BEGIN $2 spec ($3) remaining=$REM"
  bash scripts/kiro-run.sh "$1-$2-spec" "$3" spec "$out"
  git add ".kiro/specs/$2" "$f" "$out" docs/KIRO_USAGE.md docs/kiro-log/*-"$1-$2"-spec.log 2>/dev/null
  git commit -qm "kiro: $2 spec (Kiro CLI spec mode, $3)" || true
  log "$2 spec $(cat /tmp/kiro-run.status) files=$(ls .kiro/specs/$2 2>/dev/null | tr '\n' ' ')"
}
spec 34 problem-catalogue claude-haiku-5.5
spec 35 deploy-ops claude-haiku-5.5
spec 36 pack-authoring-pipeline claude-haiku-5.5
log "SPECS_DONE; resuming implementation"
# Throwaway local Postgres/Valkey for pg_/valkey_ tests. File lives outside the repo and only exports two URLs.
[ -f ~/.config/counterspell/dev-test.env ] && . ~/.config/counterspell/dev-test.env
bash scripts/kiro-implement-all.sh judge-sandbox language-drivers auth ide-workspace efficiency-familiar hints dashboard-fsrs counterspell-mcp-power landing-steel-night break-room 2>&1 | tee -a /mnt/c/Users/Brighton/kiro-impl.log
log "runner exited: $(tail -1 /tmp/kiro-impl.progress)"
