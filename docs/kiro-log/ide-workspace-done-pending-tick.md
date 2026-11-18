# ide-workspace — completed work awaiting a task tick

## T02 custom ghost-run Axum seam (18 Nov)

- `POST /api/runs/ghost` in `crates/server/src/runs/custom.rs`, routed in `lib.rs`.
  Check order mirrors `POST /api/runs`: body and content type, user, idempotency
  key, language and source validation, published problem, then the ghost checks
  (submission id parses, input <= 64 KiB serialised), the single ownership lookup,
  replay, rate limits and the atomic enqueue. Missing and foreign submissions both
  answer `404 unknown_submission`; ownership is never revealed.
- Migration `20261118100000_custom_ghost_runs.sql`: `ghost_input jsonb` and
  `ghost_source_id uuid REFERENCES submissions ON DELETE SET NULL`, bounded by
  `submissions_ghost_ck` (source implies input; input capped at 64 KiB). The
  `counterspell_server` INSERT column list was re-granted with the two columns.
- `NewRun::ghost: Option<GhostRef>` flows through `enqueue`/`existing_replay`:
  replay identity now includes the ghost input and source id, so the same key
  with a different ghost is `idempotency_key_reuse`, and the insert writes the
  columns atomically with the job row.
- Judge side: `claim` loads `ghost_input` onto `ClaimedRun` and `RunInput`;
  `SandboxExecutor::execute_ghost` runs the user source and the trusted Python
  reference in separate containers through `SandboxOracle::evaluate`, classifies
  the observation with the problem comparator and emits `compiling` plus one
  public `test` event (input/expected/got — the ghost is already the users own
  data). No bench or familiar measurement runs for a custom run. A missing
  reference is `problem_unavailable`, never a verdict.
- Response shape per the design contract: `202 {run_id, status:"queued",
  mode:"run", custom:true}` plus `Location`.
- Tests: `crates/server/tests/ghost_runs.rs` — validation table (source limits,
  bad lang, unknown fields, bad submission id, oversized input, wrong content
  type, unknown problem), unauthenticated, and pg tests for missing/foreign
  source, atomic persist, replay/reuse and the active cap (9 pass). Judge:
  `model_sandbox_executor` ghost suite — pass, wrong answer, runtime error,
  timeout and missing reference (5 pass); `pg_claim_loads_the_ghost_input`.
- Gates: `cargo fmt --check`, `clippy --workspace --all-targets --features
  sandbox -D warnings` clean; post_runs/enqueue/get_run/claim/model suites all
  green (server 32 + judge 26 tests).
- Note: D16 stays open — this path does not need a `ghosts` table because the
  request carries the ghost input and the judge re-derives the expected value
  from the trusted reference.

## T09 responsive WorkspaceShell (18 Nov)

- `layout-mode.svelte.ts`: matchMedia-driven `desktop >=1200px`,
  `tablet 768-1199px`, `narrow <768px` buckets, SSR-safe default `desktop`.
  The component takes an optional `layout` prop so tests pin a mode.
- Desktop keeps the two-column shell (grid switched from `lg:` to an explicit
  `minmax(16rem,22rem)` column pair under the mode branch so JS is the single
  source of truth for breakpoints).
- Tablet renders the editor and controls in flow with a persistent
  `role="status" aria-label="Run status"` strip (live phase label or terminal
  verdict), plus `Problem`/`Results` drawer triggers. Drawers are
  `role="dialog"` + `tabindex="-1"` panels; opening moves focus inside,
  Escape/Close returns focus to the trigger (`aria-expanded`, `aria-controls`).
- Narrow stacks problem, editor, results in reading order with the same
  collapse toggle; ProblemPanel already keeps title/constraints visible with a
  collapsible statement.
- Shared `{#snippet problemContent|resultsContent|problemAside|editor}`
  keeps all three layouts consistent; run-session wiring unchanged.
- Vitest (WorkspaceShell.test.ts, 7 tests): named landmarks, collapse/restore,
  one-run-per-click, tablet triggers/drawer focus + Escape/restore, status
  strip, narrow reading order. `ProblemWorkspace.test.ts` sets a 1440px
  happy-dom viewport (`window.happyDOM.setViewport`) since 1024px defaults to
  tablet mode.
- Browser verification (real Chrome via playwright-core channel, vite dev +
  `web/e2e-stub.mjs` fixture server, `web/e2e-shell-check.mjs` — 30 checks):
  desktop columns side-by-side, tablet drawers/focus/Escape/status strip,
  live resize swaps modes without reload, narrow full-width stack, no
  horizontal overflow at 1440/900/390 and 200% zoom (720 CSS px), min target
  44px at every breakpoint, route change loads new slug with no stale title
  and back returns. Only console error: favicon 404.
- Gates: `npm run check` 0 errors 0 warnings; vitest 465 tests green after the
  viewport fix.
- Open per design, deferred to later tasks: Familiar overlay position (T20),
  heavy-asset lazy-load measurements (T27), keyboard shortcut map (T21).

## T13 production Run/Submit against Axum (18 Nov)

- `crates/testserver`: new publish=false fixture crate running the real Axum
  router, `BearerTestResolver` identities, `RunWaker`, and a scripted
  `Worker`/`ScriptedExecutor` (`Script::Pass`, 3 tests, 30 ms) against real
  Postgres. Boot migrates, seeds 256 users and `two-sum` (idempotent). Kept
  out of `crates/server` on purpose: an optional judge dep put
  `counterspell-judge` inside the server's production closure and failed
  `boundary::the_server_cannot_reach_docker_or_the_judge` — the separate crate
  keeps the boundary green (verified 6/6).
- `run-session.axum.test.ts` (`@vitest-environment node`, skipped unless
  `AXUM_TEST_URL`): drives the unmocked `startRunAction`/`getRun`/`watchRun`
  through `fetchImpl`/SSE-adapter seams with `Bearer test-user-N`. 4 tests:
  sample run 202 -> streamed events -> terminal read model (`completed`,
  3 tests); oversized source -> server error envelope -> `failed` with prior
  view kept; second request under `RUN_RATE_USER=1` -> `rate_limited`, prior
  view kept; foreign run id -> real `404 not_found`, nothing leaked. Time-
  rotated users (of 256) keep reruns inside one limiter window deterministic
  (two consecutive suite runs both 4/4).
- Already-built pieces confirmed in place: per-action idempotency key in
  `runs/api.ts`, generation-token cancellation in `run-session.ts`,
  `view` carried across `starting`/`streaming` until the replacement is
  terminal, per-language localStorage drafts in `editor-session.ts`.
- Gates: `cargo fmt --check`, `clippy -D warnings` (testserver), `npm run
  check` 0/0, axum suite 4/4 twice, `boundary` 6/6.
- Not covered here by design: full browser journey on real envelopes (T24/T25)
  and reconnect storms (T26).

## T15 — Ghost Hunting + Ghost card (done pending tick)

- GhostPanel and AutopsyCard existed but were mounted nowhere; the stream
  reducer already carried ghost/shrink_step data that never reached the DOM.
- WorkspaceShell now mounts both cards inside the Results region, fed from
  `runs.stream` (status, ghost, shrinkSteps). Ghost data persists after the
  stream finishes because the terminal stream state is retained.
- New seams: `sessionDeps` prop (injects RunSessionDeps for tests, defaults to
  browserRunSessionDeps) and `reducedMotion` prop feeding
  `reduced-motion.svelte.ts`, a live prefers-reduced-motion tracker.
- Both prop reads are one-time captures wrapped in `untrack` — keeps
  svelte-check at zero warnings without svelte-ignore comments.
- Tests: WorkspaceShell.test.ts gained a shell-level test that runs the real
  createRunSession against fake deps replaying a ghost-carrying stream and
  asserts the card + recorded steps inside the Results region. GhostPanel.test.ts
  already covered hunting text (no percentages), minimized wording, timeout not
  minimized, all actual steps in order, reduced-motion static mode,
  disagreeing steps hidden, cut-off, malformed and no-ghost states.
- Prototype banner: /prototype/ide-workspace already renders PROTOTYPE_BANNER
  in a visible bordered block above the fixture layout.
- Gates: `npm run check` 0 errors / 0 warnings; workspace vitest 96 passed
  (4 skipped = axum suite without env); GhostPanel 11/11; shell 8/8.

## T18 — Rerun ghost as a real custom run (done pending tick)

- `runs/api.ts`: `startGhostRun`/`startGhostRunAction` POST `/api/runs/ghost`
  with `{problem, lang, code, ghost:{input}}` and one Idempotency-Key per
  action, same contract as `startRunAction`.
- `run-session.ts`: `SessionStartInput.ghostInput` routes the start through
  `deps.ghostAction` (optional dep; absent = start refused, returns false).
  A refused send now restores `stream` (captured before the starting publish)
  so the autopsy of the finished run is not erased by an API error.
- `RunControls`: labelled `Rerun ghost` button (44px, title explains it),
  rendered only when `canRerunGhost` and wired through `EditorPanel` as a
  `RunRequest{ghost:true}` carrying the live editor source.
- `WorkspaceShell`: `canRerunGhost` = current stream has a ghost; the shell
  supplies `runs.stream.ghost.input` so the judge gets the stored input
  verbatim. The browser can never invent the input.
- RED to GREEN: api.test (endpoint, body, idempotency), run-session.test
  (ghost action used, prior stream+view preserved on refusal),
  RunControls.test (hidden without ghost, labelled click),
  WorkspaceShell.test (full path: ghost stream to button to ghostAction
  called with problem/lang/ghost input).
- Gates: workspace+runs vitest 147 passed (4 skipped axum-env); `npm run
  check` 0 errors, 0 warnings.
- Not covered here: Axum-level ghost rerun (the endpoint itself is covered
  by server pg tests from T02); browser-harness rerun journey (T24).
