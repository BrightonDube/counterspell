# Run-lifecycle blockers

Written by the implementation run on 13 Nov 2026. Each entry names the task, the reason it is not finished, and what would unblock it.

## Environment: PreToolUse hook blocks some test writes (T02, T03)

- **What happened:** the file-write tool was blocked by a PreToolUse hook several times. The hook returned exit code 2 with no reason. The blocks were not tied to the test-integrity guard, which only counts removed tests and added skip markers, and that guard passes when run by hand with a bare path. Blocks came and went with content. The `FailureReason` tests were blocked every time they were tried, while the enum and its message table went through. The blocks were not bypassed (no shell writes were used to get around them).
- **Tasks affected:** T02 (the `FailureReason` round-trip and distinct-message tests are not written). T03 (`status_after_event`, `terminal_event_kind` and the proptests are not written, because they depend on the event types in T04).
- **Unblock:** Brighton to check which PreToolUse hook is rejecting writes in this workspace (`.kiro/hooks/` and any global hooks), and whether the rejection can be logged with a reason. Then the missing tests can be added as their own commit.

## T02: `FailureReason` tests missing

- The enum and `public_message()` exist in `crates/common/src/status.rs`. There are no tests for them yet, so T02 stays unticked.

## T03: state machine partly done

- Done: `can_transition`, `job_state_for`, five unit tests (`crates/common/src/run_state.rs`).
- Not done: `status_after_event` and `terminal_event_kind` (they need `PublicEvent`, which is T04), and the P14 proptests. T03 stays unticked.

## Not started: T04 onward

- T04 to T39 were not started in this run. The list is long, and the hook problem above was blocking test writes, so the tasks were not attempted.
