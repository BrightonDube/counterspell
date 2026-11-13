#!/usr/bin/env bash
# Installs the gitleaks pre-commit and pre-push hooks into this clone (.git/hooks is shared by all worktrees).
set -euo pipefail
root=$(git rev-parse --show-toplevel)
hooks=$(git rev-parse --git-common-dir)/hooks
for h in pre-commit pre-push; do
  install -m755 "$root/scripts/git-hooks/$h" "$hooks/$h"
  echo "installed $hooks/$h"
done
command -v gitleaks >/dev/null || [ -x "$HOME/.local/bin/gitleaks" ] || echo "Now install gitleaks: https://github.com/gitleaks/gitleaks#installing"
