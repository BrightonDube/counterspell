# ide-workspace T18–T25: hint ladder mount and capture-phase run shortcuts

Mounts the authored hint ladder in WorkspaceShell, fixes the hint response field (`text` to `body`), and moves the Ctrl/Cmd(+Shift)+Enter listener to the window capture phase. Adds a ghost-rerun rate-limit pg test, cap-fallback and remount tests, and real-browser scripts.
Watch for: the ladder's stale-response guard has no test, and the shortcut now swallows Ctrl+Enter anywhere outside input/select, including the hint panel and any open dialog (likely).

**Verdict**: APPROVED

## High-level view
The ladder is keyed to the finished run's id. An effect resets it on a new id, and a late reply is dropped unless its run id still matches. The logic is sound, but only the happy and error paths are tested.

The shortcut listener now runs first in the capture phase and consumes any recognised key, even while busy. That is needed because Monaco stops the event at its own element. The side effect is that Ctrl+Enter is swallowed outside the editor too.

The `body` fix aligns the client with the server's HintView and is covered by a marked test change.

<details>
<summary>Issues (5)</summary>

1. **Stale-response guard untested (Important, confirmed)**: no test starts a request for run A, finishes run B, then resolves A. Add one asserting the ladder is untouched and no hint text for A appears.
2. **Ladder reset untested (Important, confirmed)**: no test checks that a second finished run clears revealed hints and the error line.
3. **Run id read from non-reactive `ladderRunId` (Minor, likely)**: it is set in an `$effect`, so it lags `runs.runId` by one flush. A click between the run finishing and the effect running sees `''` and silently returns after `requestNext` has set `pending`, leaving rung 1 stuck pending. Use `runs.runId` directly, or call `requestNext` only after the id check.
4. **Rung count hard-coded to 4 (Minor, confirmed)**: matches `HintRung` (1 to 4) in `crates/common/src/hints.rs`, but is a second copy. Server returns `next_rung: None` after 4; prefer deriving from that or a shared constant. Not blocking.
5. **Shortcut consumption scope (Minor, likely)**: only input, select and non-Monaco textarea are exempt. Ctrl+Enter inside a Bits UI dialog/popover or a focused button is swallowed and triggers Run. Consider also exempting `[role=dialog]`. Consuming while busy is fine: it only prevents Monaco inserting a blank line.
</details>

<details>
<summary>Details</summary>

### Ladder race handling
`onNextHint` captures `ladderRunId` before awaiting and compares after, so a reply for an old run cannot write the new ladder. `receiveRung` and `failRung` additionally ignore any rung that is not `pending`. The failure path uses a fixed message, so server text never reaches the UI; the test asserts that. While a new run is in flight the panel is hidden but `ladderRunId` still holds the old id, so a late reply mutates a hidden ladder. That is harmless because the next finished run resets it.

### Capture-phase shortcut
Window capture plus `stopPropagation` guarantees the key beats Monaco's insert-line binding. The new RunControls test simulates Monaco with a stop-propagation listener and asserts the editor sees nothing, which is a real check of the behaviour, not a tautology. `isFormField` still lets inputs and selects keep Ctrl+Enter. IME composition is not handled (`event.isComposing`); Enter during composition with Ctrl is rare, so Minor.

### Tests
The remount test asserts exactly one POST path across remount, and the ghost_runs pg test asserts 429 `rate_limited` and no queued job. Both check observable behaviour. The `test-change: HNT-1` marker on hints.test.ts is justified: the server contract sends `body`.
</details>

<details>
<summary>File map</summary>

- crates/server/tests/ghost_runs.rs: rerun shares rate limit
- web/src/lib/workspace/WorkspaceShell.svelte: hint ladder mount, `requestHint` prop
- web/src/lib/workspace/RunControls.svelte: capture-phase shortcuts
- web/src/lib/api/hints.ts: read `body`
- web/src/lib/**/*.test.ts: new unit tests
- web/tests/real-api/*.mjs: real-stack browser checks

Full diff: `git diff 434e7a86..dde26abf`
</details>
