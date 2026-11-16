#!/usr/bin/env bash
# Publish check for pushed commits (pack-authoring-pipeline PA17; PIP-28, PIP-44; PUB-01).
#
# Usage: scripts/pack-publish-check.sh <git-range>        (for example origin/main..HEAD)
#        scripts/pack-publish-check.sh <lsha> --not --remotes   (a new branch, as pre-push passes it)
#
# For each commit in the range, if it changes a pack's meta.toml so that `status = "published"`
# is newly present, the commit message must contain "Approved by Brighton". Otherwise the check
# prints PUB-01 and exits 1. It reads git history only and never pushes or changes configuration.
set -euo pipefail

if [[ $# -lt 1 ]]; then
  printf '%s\n' 'usage: pack-publish-check.sh <git-rev-list arguments...>' >&2
  exit 2
fi
range=("$@")
phrase='Approved by Brighton'
status=0

# Every commit in the range, oldest first, so the messages print in push order.
for commit in $(git rev-list --reverse "${range[@]}"); do
    # Meta files this commit changed, that exist after the commit.
    for meta in $(git diff-tree --no-commit-id --name-only -r "$commit" -- 'packs/*/meta.toml' 2>/dev/null); do
        now="$(git show "$commit:$meta" 2>/dev/null || true)"
        before="$(git show "$commit^:$meta" 2>/dev/null || true)"
        if ! grep -Eq 'status[[:space:]]*=[[:space:]]*"published"' <<< "$now"; then
            continue
        fi
        if grep -Eq 'status[[:space:]]*=[[:space:]]*"published"' <<< "$before"; then
            continue
        fi
        if ! git log -1 --format=%B "$commit" | grep -Fq -- "$phrase"; then
            printf 'PUB-01: commit %s publishes %s without "%s"; not pushed\n' "$commit" "$meta" "$phrase" >&2
            status=1
        fi
    done
done
exit "$status"
