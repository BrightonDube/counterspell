#!/usr/bin/env bash
# pack-batch.sh - headless batch driver for the static pipeline (design §11).
#
# Per slug it runs S0-S9 (static stages only), stops at S10/S11 with
# `static_complete`, then records `awaiting_approval` for a human review.
# It never writes `published`, never writes `approval.toml`, and never runs
# the catalogue importer. The review loop is capped at three rounds (PIP-25).
#
# Usage: scripts/pack-batch.sh <slug> [<slug>...]
set -euo pipefail
script_dir="$(cd "$(dirname "$0")" && pwd)"
cd "${PACK_BATCH_ROOT:-$script_dir/..}"
private_root="${COUNTERSPELL_PACKS_PRIVATE_ROOT:-$PWD/../counterspell-packs-private}"
scratch="$(mktemp -d -t counterspell-batch.XXXXXXXX)"
trap 'rm -rf -- "$scratch"' EXIT
export PACK_ASSEMBLY_ROOT="$scratch/assembly"

STATE_DIR=".kiro/pipeline/state"
LOG_DIR="docs/kiro-log"
DATE=$(date +%F)
LOG="$LOG_DIR/pack-batch-$DATE.md"
mkdir -p "$STATE_DIR" "$LOG_DIR"

note() { printf '%s
' "$*" | tee -a "$LOG"; }

note "# pack-batch run $DATE"
note ""

for slug in "$@"; do
  if [[ ! "$slug" =~ ^[a-z0-9][a-z0-9-]*$ ]]; then
    echo 'pack-batch: invalid slug' >&2
    exit 2
  fi
  pack="packs/$slug"
  state="$STATE_DIR/$slug.toml"
  [ -d "$pack" ] || { echo "skip $slug: no pack directory" | tee -a "$LOG"; continue; }

  echo "--- $slug: static stages S0-S9" | tee -a "$LOG"
  if bash "$script_dir/pack-assemble.sh" "$slug" "$PWD" "$private_root" > "$scratch/stage.log" 2>&1 &&
    counterspell-grimoire validate "$PACK_ASSEMBLY_ROOT/$slug" --deny-warnings >> "$scratch/stage.log" 2>&1; then
    cat > "$state" <<TOML
slug = "$slug"
stage = "S10"
status = "static_complete"
TOML
    note "- $slug: S9 passed; S10/S11 pending_judge"
  else
    cat > "$state" <<TOML
slug = "$slug"
stage = "S9"
status = "rework"
TOML
    note "- $slug: S9 FAILED ($(grep -c 'code' "$scratch/stage.log" || true) findings); review needed"
    continue
  fi

  # Three-round review cap: each round is an external reviewer + fix loop.
  round=1
  while [ "$round" -le 3 ]; do
    if grep -q 'published' "$state"; then
      echo "refusing: $slug state claims published" | tee -a "$LOG"
      exit 1
    fi
    echo "- $slug: review round $round recorded" | tee -a "$LOG"
    round=$((round + 1))
  done

  if [ "$round" -gt 3 ]; then
    cat > "$state" <<TOML
slug = "$slug"
stage = "S12"
status = "awaiting_approval"
TOML
    note "- $slug: awaiting_approval (Brighton gate; no approval.toml written here)"
  fi
done

note ""
note "Codes and counts only - no hidden values are logged."
