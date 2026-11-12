#!/usr/bin/env bash
# Pushes brighton/kiro-setup to GitHub every 10 minutes while Kiro commits.
cd "$(dirname "$0")/.."
while sleep 600; do
  [ "$(git branch --show-current)" = brighton/kiro-setup ] || continue
  git push -q origin brighton/kiro-setup >> /tmp/kiro-autopush.log 2>&1 && echo "$(date '+%F %H:%M') pushed $(git rev-parse --short HEAD)" >> /tmp/kiro-autopush.log
done
