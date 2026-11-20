#!/usr/bin/env bash
set -euo pipefail

file_path=${1:-}
event=$(cat)
if [[ -z "$file_path" ]] && command -v jq >/dev/null 2>&1; then
  file_path=$(printf '%s' "$event" | jq -r '.filePath // .path // ""' 2>/dev/null || true)
fi
if [[ -z "$file_path" ]]; then
  printf '%s\n' 'skipped: no changed pack path was supplied'
  exit 0
fi
# LAY-07: a private file must never be saved under packs/ (its public tree or the working copy).
case "$file_path" in
  */hidden.toml|hidden.toml|*/generator.toml|generator.toml|*/known_ghosts.toml|known_ghosts.toml|*/approval.toml|approval.toml|*/retired-values.toml|retired-values.toml)
    if [[ "$file_path" == packs/* || "$file_path" == */packs/* ]]; then
      printf 'LAY-07: private file saved in the public packs tree: %s\n' "$file_path"
      exit 2
    fi
    ;;
esac
if [[ ! -f "$file_path" ]]; then
  printf 'skipped: changed pack file does not exist: %s\n' "$file_path"
  exit 0
fi
if [[ "$file_path" != packs/* && "$file_path" != */packs/* ]]; then
  printf 'skipped: changed file is outside packs: %s\n' "$file_path"
  exit 0
fi
if ! command -v cargo >/dev/null 2>&1; then
  printf '%s\n' 'skipped: cargo is not installed'
  exit 0
fi
if [[ ! -f Cargo.toml ]]; then
  printf '%s\n' 'skipped: Cargo.toml does not exist'
  exit 0
fi
if [[ ! -d crates/grimoire ]]; then
  printf '%s\n' 'skipped: crates/grimoire does not exist'
  exit 0
fi

if [[ "$file_path" == packs/* ]]; then
  pack_path=${file_path#packs/}
else
  pack_path=${file_path##*/packs/}
fi
slug=${pack_path%%/*}
if [[ -z "$slug" || ! -d "packs/$slug" ]]; then
  printf 'skipped: changed pack directory does not exist for %s\n' "$file_path"
  exit 0
fi
cargo run -q -p counterspell-grimoire -- validate "packs/$slug"
