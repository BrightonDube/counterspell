#!/usr/bin/env bash
# Local pre-push gate. Used by scripts/git-hooks/pre-push (core.hooksPath).
# Order: gitleaks on the commits being pushed, then Rust fmt/clippy/test,
# then web lint, svelte-check, Vitest and the production build.
# Any failure aborts the push (exit 1). Compatible with
# /home/brighton/kiro-push-after-spec.sh, which already runs tests then
# `git push` as Brighton; this hook runs again during that push.
set -euo pipefail

root=$(git rev-parse --show-toplevel)
cd "$root"

# --- gitleaks (reads stdin: local_ref local_sha remote_ref remote_sha) ---
GL=$(command -v gitleaks || true)
if [ -z "$GL" ] && [ -x "$HOME/.local/bin/gitleaks" ]; then
  GL="$HOME/.local/bin/gitleaks"
fi
zero=0000000000000000000000000000000000000000
pushing=0
if [ -n "$GL" ] && [ -x "$GL" ]; then
  cfg=()
  [ -f "$root/.gitleaks.toml" ] && cfg=(--config "$root/.gitleaks.toml")
  status=0
  while read -r _lref lsha _rref rsha; do
    [ "$lsha" = "$zero" ] && continue
    pushing=1
    if [ "$rsha" = "$zero" ]; then
      range="$lsha --not --remotes"
    else
      range="$rsha..$lsha"
    fi
    "$GL" git "${cfg[@]}" --log-opts="$range" --redact --no-banner --exit-code 1 "$root" || status=1
  done
  if [ "$status" -ne 0 ]; then
    echo "Push blocked: gitleaks found a possible secret. Rotate it, remove it, then push." >&2
    exit 1
  fi
else
  echo "WARNING: gitleaks not installed; pre-push secret scan skipped (CI will still scan)." >&2
  while read -r _lref lsha _rref _rsha; do
    [ "$lsha" = "$zero" ] && continue
    pushing=1
  done
fi

if [ "$pushing" -eq 0 ]; then
  exit 0
fi

echo "pre-push: cargo fmt --check"
cargo fmt --all -- --check

echo "pre-push: cargo clippy"
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

echo "pre-push: cargo test --workspace"
cargo test --workspace --all-features --locked

echo "pre-push: web lint"
npm --prefix web run lint

echo "pre-push: web svelte-check"
npm --prefix web run check

echo "pre-push: web vitest"
npm --prefix web run test

echo "pre-push: web build"
npm --prefix web run build

echo "pre-push: all checks passed"
