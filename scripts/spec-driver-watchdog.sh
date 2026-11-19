#!/usr/bin/env bash
set -u
WT="/home/brighton/cs-queue"
LOG="$WT/docs/kiro-log/spec-driver-runs.md"
CMD="drive queue — this run works on branch kiro/drive-queue in worktree /home/brighton/cs-queue; Brighton directed the clean-branch run, so the brighton/kiro-setup branch check in your prompt is satisfied by kiro/drive-queue for this session. Note: Playwright chromium is installed in web/ and cs-dev-pg is freshly migrated."
cd "$WT" || exit 1
if tmux has-session -t spec-driver 2>/dev/null; then
  exit 0
fi
OPEN=$(grep -hE '^\s*- \[ \] \*\*' .kiro/specs/*/tasks.md 2>/dev/null | grep -vE 'HUMAN|stretch|BLOCKED' | wc -l)
if [ "$OPEN" -eq 0 ]; then
  echo "$(date '+%F %H:%M') watchdog: queue idle and no open agent tasks; not restarting" >> "$LOG"
  exit 0
fi
echo "$(date '+%F %H:%M') watchdog: spec-driver exited; $OPEN open task lines remain, restarting queue" >> "$LOG"
bash scripts/spec-driver.sh start "$CMD" 480 >/dev/null 2>&1 || true
