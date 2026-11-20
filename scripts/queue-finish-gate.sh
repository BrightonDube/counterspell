#!/usr/bin/env bash
# Runs every 5 minutes after the queue watchdog stops seeing open agent tasks.
# Once the queue is no longer running and no open tasks remain, it commits the
# watchdog script + any leftover docs fixes, runs the full verification gate,
# pushes kiro/drive-queue, and stops scheduling itself.
set -u
WT="/home/brighton/cs-queue"
cd "$WT" || exit 1

# If spec-driver is still alive, wait.
if tmux has-session -t spec-driver 2>/dev/null; then
  exit 0
fi

# If any open non-HUMAN task lines remain, the watchdog will restart the queue.
OPEN=$(grep -hE '^\s*- \[ \] \*\*' .kiro/specs/*/tasks.md 2>/dev/null | grep -vE 'HUMAN|stretch|BLOCKED' | wc -l)
if [ "$OPEN" -gt 0 ]; then
  exit 0
fi

# Queue is done. Commit leftover edits (watchdog, docs logs, test fixture fixes).
echo "$(date '+%F %H:%M') gate: queue complete, running final gate" >> docs/kiro-log/spec-driver-runs.md
git add scripts/spec-driver-watchdog.sh docs/kiro-log/ crates/testserver/src/main.rs web/src/lib/workspace/run-session.axum.test.ts 2>/dev/null || true
git commit -m "chore: watchdog, docs log, testserver fixture sizing for rerun stability" 2>/dev/null || true

# Full gates
set -e
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --locked
cd web && npm run lint && npm run check && npm run test && npm run build && cd ..

# Push the branch
git push -u origin kiro/drive-queue
echo "$(date '+%F %H:%M') gate: pushed kiro/drive-queue" >> docs/kiro-log/spec-driver-runs.md
