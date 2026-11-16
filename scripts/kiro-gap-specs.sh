#!/usr/bin/env bash
# Writes the specs for the gaps found on 14 Nov 2026 (spec mode only, no implementation),
# then amends problem-catalogue for the private-pack split (38),
# then runs docs/kiro-prompts/37-remove-hackathon-mentions.md as its own default-mode session,
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

# Amend the already-written problem-catalogue spec for the private-pack split
# (Brighton, 14 Nov 23:06). Spec only. Skips if the split is already in the spec.
amend_private_pack_split(){
  local f=docs/kiro-prompts/38-private-pack-split.md
  [ -f "$f" ] || { log "STOPPED missing $f"; exit 1; }
  if [ -f .kiro/specs/problem-catalogue/design.md ] && grep -q 'counterspell-packs-private' .kiro/specs/problem-catalogue/design.md .kiro/specs/problem-catalogue/requirements.md 2>/dev/null; then
    log "38-private-pack-split already reflected in problem-catalogue, skipped"
    return 0
  fi
  local U REM; U=$(usage); [ -n "$U" ] || { log "STOPPED could not read credits"; exit 1; }
  REM=$(awk -v u="$U" -v l=$LIMIT 'BEGIN{printf "%.0f", l-u}')
  [ "$REM" -ge "$SPEC_FLOOR" ] || { log "STOPPED remaining credits $REM < $SPEC_FLOOR before 38-private-pack-split"; exit 1; }
  log "BEGIN 38-private-pack-split ($1) remaining=$REM"
  bash scripts/kiro-run.sh "38-private-pack-split" "$1" spec "$f"
  git add .kiro/specs/problem-catalogue "$f" docs/KIRO_USAGE.md docs/kiro-log/*-38-private-pack-split.log 2>/dev/null
  git commit -qm "kiro: amend problem-catalogue for the private-pack split (Kiro CLI spec mode, $1)" || true
  log "38-private-pack-split $(cat /tmp/kiro-run.status)"
}
amend_private_pack_split claude-haiku-5.5

# Neutralize event-specific wording (Brighton, 14 Nov 2026) after the three specs
# and before implementation resumes. Own default-mode session, not a new spec.
remove_event_wording(){ # model
  local f=docs/kiro-prompts/37-remove-hackathon-mentions.md
  [ -f "$f" ] || { log "STOPPED missing $f"; exit 1; }
  local U REM; U=$(usage); [ -n "$U" ] || { log "STOPPED could not read credits"; exit 1; }
  REM=$(awk -v u="$U" -v l=$LIMIT 'BEGIN{printf "%.0f", l-u}')
  [ "$REM" -ge "$SPEC_FLOOR" ] || { log "STOPPED remaining credits $REM < $SPEC_FLOOR before remove-hackathon-mentions"; exit 1; }
  log "BEGIN remove-hackathon-mentions ($1) remaining=$REM"
  bash scripts/kiro-run.sh "37-remove-hackathon-mentions" "$1" default "$f"
  git add docs/kiro-prompts/37-remove-hackathon-mentions.md docs/KIRO_USAGE.md docs/kiro-log/remove-hackathon-mentions.md docs/kiro-log/*-37-remove-hackathon-mentions.log 2>/dev/null
  git commit -qm "kiro: remove-hackathon-mentions run log (Kiro CLI, $1)" || true
  log "remove-hackathon-mentions $(cat /tmp/kiro-run.status)"
}
remove_event_wording claude-haiku-5.5
log "SPECS_DONE; resuming implementation"
# Throwaway local Postgres/Valkey for pg_/valkey_ tests. File lives outside the repo and only exports two URLs.
[ -f ~/.config/counterspell/dev-test.env ] && . ~/.config/counterspell/dev-test.env
bash scripts/kiro-implement-all.sh judge-sandbox language-drivers auth ide-workspace efficiency-familiar hints dashboard-fsrs counterspell-mcp-power landing-steel-night break-room 2>&1 | tee -a /mnt/c/Users/Brighton/kiro-impl.log
log "runner exited: $(tail -1 /tmp/kiro-impl.progress)"
