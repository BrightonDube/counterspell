#!/usr/bin/env bash
# Writes every remaining spec (no implementation). Usage: bash scripts/kiro-write-all-specs.sh
cd "$(dirname "$0")/.."
run(){ # num slug model
  local f=docs/kiro-prompts/$1-$2.md out=docs/kiro-prompts/$1-$2-cli.md
  { sed -n '/^```$/,/^```$/p' "$f" | sed '1d;$d' | sed 's/#\[\[file:\([^]]*\)\]\]/\1/g'
    printf '\nThis run is non-interactive (Kiro CLI). Read the referenced docs and .kiro/steering/ yourself. Write the complete spec (requirements.md, design.md, tasks.md) under .kiro/specs/%s/ in one go, then stop. This is the system-design stage: do NOT implement any task, and do not create or change any file outside .kiro/specs/%s/. Keep consistent with the existing .kiro/specs/repo-skeleton/ (layout, crate names). Put open questions for Brighton at the end of design.md under "Open questions", each with the choice you assumed.\n' "$2" "$2"; } > "$out"
  bash scripts/kiro-run.sh "$1-$2-spec" "$3" spec "$out"
  git add -A && git commit -qm "kiro: $2 spec (Kiro CLI spec mode, $3)"
  echo "$(date +%H:%M) $(cat /tmp/kiro-run.status)" >> /tmp/kiro-specs.progress
}
: > /tmp/kiro-specs.progress
run 04 ghost-engine claude-sonnet-5.5
run 05 grimoire-packs claude-haiku-5.5
run 06 judge-sandbox claude-sonnet-5.5
run 07 run-lifecycle claude-sonnet-5.5
run 08 auth claude-sonnet-5.5
run 09 ide-workspace gpt-5.6-luna
run 10 language-drivers gpt-5.6-luna
run 11 hints claude-haiku-5.5
run 12 dashboard-fsrs gpt-5.6-luna
run 13 counterspell-mcp-power gpt-5.6-luna
run 14 efficiency-familiar claude-sonnet-5.5
run 15 landing-steel-night claude-haiku-5.5
run 16 break-room claude-haiku-5.5
echo ALL_DONE >> /tmp/kiro-specs.progress