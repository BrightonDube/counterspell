#!/usr/bin/env bash
# Point this clone at the tracked hooks in scripts/git-hooks (shared by all worktrees).
# pre-push runs scripts/pre-push.sh: gitleaks, cargo fmt/clippy/test, then web lint,
# svelte-check, Vitest and the build. See docs/ci-cd.md.
set -euo pipefail
root=$(git rev-parse --show-toplevel)
git -C "$root" config core.hooksPath scripts/git-hooks
echo "set core.hooksPath=scripts/git-hooks"
command -v gitleaks >/dev/null || [ -x "$HOME/.local/bin/gitleaks" ] || echo "Now install gitleaks: https://github.com/gitleaks/gitleaks#installing"
