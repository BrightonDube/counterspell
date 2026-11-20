#!/usr/bin/env bash
# H-ci: fail if a configured AI-provider key pattern is committed.
set -euo pipefail
pattern="sk-[A-Za-z0-9_-]{16,}|gsk_[A-Za-z0-9_-]{16,}|sk-ant-[A-Za-z0-9_-]{16,}|AIza[0-9A-Za-z_-]{20,}"
if git grep -nI -E "$pattern" -- . ":!scripts/check-no-hint-secrets.sh" ":!*.md" | grep -vE "sk-(test|fake|example|fixture)|gsk_(test|fake|example|fixture)|sk-secret-value|your-|placeholder|xxxxx"; then
    echo "hint-secret check FAILED: provider key pattern in tracked files" >&2
    exit 1
fi
echo "hint-secret check OK"
