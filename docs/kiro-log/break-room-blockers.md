# break-room blockers and gate note

Run: break-room implement, attempt 1 (16 Nov 2026).

## T00 gate note: FAIL

T00 says break room must not start while the freeze-safe core is incomplete. It also says: if a check fails, stop and do not start T01. Result: no task from T01 to T20 was started, and no code was changed.

| Check | Result | Evidence |
|---|---|---|
| Real fail → ghost flow verified locally (Docker on this PC) | FAIL | `ide-workspace/tasks.md` T24–T28 are unchecked. These are the real-flow proof tasks (T24 real Axum browser harness, T25 judge journey without reload, T26 reconnect and failure paths, T28 full verification gate). The acceptance line "A real wrong submission enters Ghost Hunting…" is not ticked. |
| Postgres queue and SSE replay work | PARTIAL | `run-lifecycle` has 54 of 56 tasks ticked. The two open items are the P1–P18 property test sign-off and the CI-green line (line 419 and 421 of its tasks.md). |
| Auth spec complete | FAIL | `auth/tasks.md` has 21 unchecked tasks, including T33 (wire server, Compose and Caddy), T34 (connect to run-lifecycle, end-to-end) and T37 (final verification and manual smoke). The manual smoke with real GitHub, Google and Resend is also unchecked. |
| IDE spec complete | FAIL | `ide-workspace/tasks.md` has 25 unchecked tasks (T02, T09, T13, T15, T18, T20–T28 and others). |
| Dashboard spec complete (core) | FAIL | `dashboard-fsrs/tasks.md` T10 (core dashboard smoke path) is unchecked. The open stretch tasks T11–T18 do not block the core. |
| Ghost engine complete | PASS | `ghost-engine/tasks.md`: 37 ticked, 0 open. |

Decision: T00 fails. Do not start T01. Brighton should decide whether to finish the freeze-safe core first or to cut break room (the spec says break room is the first feature cut). This run did not change that decision.

Unblock: tick T24–T28 of `ide-workspace` with real evidence, finish `auth` T33, T34 and T37, and close `dashboard-fsrs` T10. Then rerun T00.

## Deferred and unblocked-later notes

- All break-room tasks depend on T00, so every task T01–T20 is blocked by this gate. Nothing else in the spec can proceed without a Brighton decision.
- T02 (Lichess CSV intake) also depends on T00. It is a licence README task, so it can run once the gate passes.
- Stretch note: the spec marks every task `[stretch]`. The gate note above is the only blocker that applies to this run.
