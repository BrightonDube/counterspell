#!/usr/bin/env bash
# After the batch: re-run any spec whose tasks.md is missing (e.g. model throttling). Tries Sonnet 5.5, then Sonnet 5.
cd "$(dirname "$0")/.."
until grep -q ALL_DONE /tmp/kiro-specs.progress; do sleep 30; done
for f in docs/kiro-prompts/{04..16}-*-cli.md; do
  slug=$(basename "$f" -cli.md | cut -d- -f2-); num=$(basename "$f" | cut -d- -f1)
  for m in claude-sonnet-5.5 claude-sonnet-5; do
    [ -f ".kiro/specs/$slug/tasks.md" ] && break
    bash scripts/kiro-run.sh "$num-$slug-spec-retry" "$m" spec "$f"
    git add -A && git commit -qm "kiro: $slug spec retry ($m)"
    echo "$(date +%H:%M) RETRY $(cat /tmp/kiro-run.status)" >> /tmp/kiro-specs.progress
  done
done
echo RETRIES_DONE >> /tmp/kiro-specs.progress