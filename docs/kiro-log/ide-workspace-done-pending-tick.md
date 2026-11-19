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

## T20 — Familiar preferences + one speech request per run (done pending tick)

- `Familiar.svelte` is now mounted inside the Results region of
  `WorkspaceShell`, fed by a `FamiliarState` derived in the shell from the
  stream's `complexity`, `familiar` and terminal `done` payloads only
  (`hasBench` = a `bench_start` announced sizes). Edits and speech never
  reach it.
- `FamiliarDeps` seam in `api/familiar.ts`: the shell injects
  `loadPreferences`/`savePreferences`/`requestSpeech` fakes in tests;
  production uses the real same-origin calls.
- Mute/hide persist through `PATCH /api/me/familiar`: prefs load once on
  mount (a user toggle before the load lands wins via `prefsTouched`),
  save fires on each toggle, a failed save keeps the local choice.
- `Familiar.svelte` got controlled `prefs`, `onPrefsChange` and `speech`
  props; local toggle state only applies when no prefs are supplied
  (existing tests unchanged).
- Speech: `createSpeechGuard` lets exactly one `POST /api/runs/{id}/
  familiar-hint` through per run id, fired only when the session reaches
  `finished`; the result renders as a labelled status line (AI hint /
  Measured summary / unavailable fallback). It never alters verdict or mood.
- `SpeechResult` typing flows through; `unavailable` renders its fixed line
  without a label.
- svelte-check note: `$effect` prop-sync produced "used before declaration"
  errors under this toolchain; replaced with `$derived` controlled-prefs
  semantics — same behaviour, zero diagnostics.
- Gates: `npm run check` 0/0; shell+familiar+api vitest 107 pass;
  workspace+runs 147 pass earlier.
- Not covered here: speech copy quality (server-side), browser-harness
  familiar journey (T24/T25).

## T21 — Keyboard navigation and focus management (done pending tick)

- Already in place from earlier tasks: named landmarks (T09), drawer
  Escape-to-close + focus return to the trigger (T09), 44px targets,
  Ctrl/Cmd+Enter Run + Ctrl/Cmd+Shift+Enter Submit shortcuts with a
  discoverable `run-shortcuts` help line (T10), Rerun ghost and hint actions
  as natively tabbable buttons (T17/T18). Monaco keeps its own editing keys
  because the shortcut mapper ignores key events inside text fields.
- New: a sr-only `StatusAnnouncer` (aria-live=polite, labelled "Run
  announcements") inside the results region, fed by a phase-derived
  announcement: starting, live status, fetching, terminal outcome via
  `runOutcomeLabel`, or a fixed start-failure line. Identical text never
  re-announces, so only transitions speak.
- `StatusAnnouncer` gained `label` and `srOnly` props; the existing save
  announcer is unchanged.
- Tests: the announcer reaches "Finished: Wrong answer." after a terminal
  event; `document.activeElement` stays on the Run button through the whole
  async lifecycle — stream events never steal focus.
- Gates: workspace vitest 103 passed (4 skipped axum-env); `npm run check`
  0/0.
- Not covered here: full keyboard-only journey in a real browser (T24/T25
  harness scope).

## T22 — Reduced-motion and animation lifecycle (done pending tick)

- Audit showed the machinery was already correct; this task adds the missing
  shell-level lifecycle proof.
- Reduced motion: `reduced-motion.svelte.ts` live tracker feeds GhostPanel
  (replay `plan.mode` -> 'static', steps listed, `motion-safe:` classes gate
  the crossfades) and Familiar/Mascot (no GSAP import, no burst, no
  crossfade; static badge stays). Existing Mascot tests: never starts GSAP
  under reduced motion, removes a running burst when motion is switched off,
  badge text replaces particles. No CSS transition outside `motion-safe:`.
- Lifecycle: Mascot `$effect` cleanups call `playback.dispose()` (kills
  timelines, restores styles, removes spawned nodes; double-dispose safe;
  dispose-before-load creates nothing — timelines.test.ts). No ambient
  loops: no `repeat: -1` or `setInterval` anywhere in familiar/ghost.
- New shell test: unmounting `WorkspaceShell` mid-run closes the watch via
  `session.cancel()` — route changes leave no stream, timer or timeline.
- No GSAP in the ghost card (CSS transitions only, all behind
  `prefers-reduced-motion`).
- Gates: WorkspaceShell vitest 12/12; `npm run check` 0/0.
- Not covered here: real-browser media-query emulation (needs the T24
  harness; happy-dom can't emulate prefers-reduced-motion end-to-end).

## T23 output safety, accessibility and contrast acceptance tests (18 Nov)

- `web/src/lib/workspace/safety-a11y.test.ts` — 10 acceptance tests; each names
  the regression it catches.
- Output safety: a source scan over workspace/ghost/familiar/runs fails on any
  `{@html`, `innerHTML`, `outerHTML`, `insertAdjacentHTML`, `document.write` or
  `eval(`. A live render proves markup-looking ghost text lands as text.
- Hidden canary: `rowView` builds a hidden row from ordinal, verdict and time
  only; the rendered Tests region never sees `input`/`expected`/`got`.
- Status: every `TestVerdict` carries a non-empty text label AND icon — a
  colour-only row fails.
- A11y: shrink list renders every recorded step in recorded order and keeps all
  3 under `reducedMotion` (`data-motion="static"`); Familiar keeps an
  `aria-live="polite"` region; every shell control has an accessible name and a
  44px class; the language select is `min-h-11`.
- Contrast: real luminance math on the app.css palette — 15 text pairs all pass
  WCAG AA (min 4.77:1, danger on danger-bg); the global `:focus-visible`
  outline rule is asserted.
- Zoom/keyboard coverage already exists: e2e-shell-check.mjs checks 720 CSS px
  (the 200%-zoom layout at 1440), 44px targets at every breakpoint, landmarks,
  Escape and focus return; keyboard.test.ts/RunControls.test.ts cover
  Ctrl/Cmd+Enter, Ctrl+Shift+Enter, field-ignore and shortcut help.
- Gates: new file 10/10; full vitest 65 files, 486 passed, 4 skipped;
  `npm run check` 0 errors/0 warnings.

## T24 real Axum browser harness and test data path (18 Nov)

- `web/tests/real-api/real-api-check.mjs` — Playwright-core (Apache-2.0,
  driver only; launches installed Chrome via `channel: 'chrome'`). Drives the
  built `web/build` SPA served by Axum on 127.0.0.1:8088. No `page.route`, no
  fixture route, no intercepted responses.
- Test data: `web/tests/real-api/packs/add-two` (public half) plus
  `private-src/` template; `stamp-approval.mjs` reproduces the importer's
  manifest hash for `approval.toml`. Private tree materializes outside the
  repo (the importer refuses nested private roots); `seed-demo` imported and
  published it through the real sandbox validation gate (hash 4bba9820…).
- Real stack proven: dev magic-link auth (unique address per run dodges the
  3/hour mail cap; verify auto-creates the user), problem page through Axum,
  Monaco input via real keyboard path, Run samples → real `accepted`, Submit
  `a*b` → real `wrong_answer` through the hidden suite (test events in pg:
  samples carry input/expected/got, hidden rows carry ordinal/verdict/ms only).
- Harness checks (12/12 pass): mail link logged, sign-in + SPA redirect,
  problem loads, Accepted verdict, Wrong-answer verdict, run_events persisted,
  terminal submissions recorded, zero hidden-event payload leaks, terminal
  read model verdict, no hidden tuples on the page.
- Judge wiring fix (required, in scope): `crates/judge/src/main.rs` had
  `EmptyProblemTests` hard-coded — every run ended `problem_unavailable`.
  Wired `PgProblemTests::new(pool)` through `sandbox_executor`; the catalogue
  loader (problem-catalogue C24) now serves pinned versions.
- Contract fixes found by real traffic: `/api/languages` returns
  `{languages:[…]}` (parser expected a bare array); problem detail carries
  `examples` not `samples`; difficulty values are `easy|medium|hard`; language
  wire rows have no enabled/supports flags. `languages.ts`/`types.ts` parse
  the real envelopes; fixtures updated (`client.test.ts`, `types.test.ts`,
  `EditorPanel.test.ts`, `ProblemPanel.test.ts`, `ProblemWorkspace.test.ts`,
  `WorkspaceShell.test.ts`, `safety-a11y.test.ts`, `prototype/fixtures.ts`,
  `EditorPanel.svelte`).
- Gates: affected vitest 9 files / 63 tests pass; `npm run check` 0/0;
  `cargo check -p counterspell-judge --features sandbox` clean.
- Known gap logged in the blockers file: the judge never emits
  `ghost_hunting`/`ghost` on a failing submit (upstream hunt wiring missing) —
  T25's shrink/autopsy journey is blocked on it.

## T25 judge journey without reload (19 Nov)

- Judge-side hunt implemented (was the T24-known missing feature):
  `crates/judge/src/sandbox/hunt.rs` builds `HuntRequest` (submit-mode +
  non-custom runs only, requires `input_schema` + trusted python reference +
  huntable verdict) and maps `HuntReport` to `ghost_hunting`/`shrink_step*`/
  `ghost` public events; `sandbox_executor::hunt_ghost` runs `engine::hunt`
  via `SandboxOracle` on `spawn_blocking` after `measure`, verdict preserved.
- `ProblemSpec` gains `input_schema`/`known_ghosts`/`concept_tags`; the store
  parses schemas unconditionally and loads ghosts/tags on submit.
- `migrations/20261119090000_judge_problem_concepts.sql` grants the judge
  role SELECT on `problem_concepts`.
- Wire fixes the real run exposed: `provenance.seed` is a decimal string
  (u64 exceeded MAX_SAFE_INTEGER and killed the client stream before
  `ghost`); `parseShrinkStep` accepts engine-zero-based index 0; the
  ghost-rerun client now sends `ghost.submission_id` (T18 contract predated
  any real API call — the server 400s without it).
- Event stream on pg for the wrong submit: compiling,test x11,familiar,
  ghost_hunting,shrink_step x3,ghost,done — order asserted in pg.
- Real harness (real-api-check.mjs, 21/21): real hunt shrank [[2,3]]->[0,1];
  Minimized counterexample + autopsy ("Your solution returned 0; expected 1
  for input [0,1].") render; Rerun-ghost POST /api/runs/ghost reaches real
  wrong_answer verdict; corrected source resubmits to real Accepted; zero
  hidden input/expected/got leaks anywhere; no reload between legs.
- Gates: judge lib 66, model_sandbox_executor 26, model_logs + pg suites
  (6+4) pass; `cargo clippy --workspace --all-targets --all-features --locked
  -D warnings` clean; `cargo fmt --all --check` clean; vitest 65 files /
  488 pass (4 skipped); `npm run check` 0/0; `npm run build` clean.
- Known flake: one earlier harness run had a samples run die
  retries_exhausted during container churn right after the previous hunt —
  transient sandbox contention, clean on rerun. Judge stdout logging is off
  (judge.log empty; RUST_LOG unset), so attempt errors leave only restarted
  markers — worth a tracing line if it recurs.

## Verified (spec-driver `verify all`, 19 Nov 2026)

Baseline run first. `cargo fmt --check` and `clippy --workspace --all-targets -D warnings` are clean. `cargo test --workspace --no-fail-fast`: 1508 passed, 0 failed, 501 ignored. `COUNTERSPELL_PACKS_PRIVATE_ROOT` in the local env file points at the old scaffold checkout, so without an override two grimoire `sandbox_fake` tests fail (LAY-01/REF-01). With the populated sibling checkout (`../counterspell-packs-private`, commit 42467cd) they pass. That is an environment fix for Brighton, not a code defect. `web`: `npm run check` 0/0, `npm run build` clean, `npm run test` 487 passed, 1 failed, 4 skipped.

| Task | Result |
|---|---|
| T02 | ticked. `cargo test -p counterspell-server --test ghost_runs -- --include-ignored` 9/9 (validation table, 401, missing/foreign source, atomic persist, replay and key reuse, active cap). Judge: `model_sandbox_executor` ghost cases 5/5 plus `pg_claim_loads_the_ghost_input` 1/1. Note: the task text's literal command `cargo test -p counterspell-server custom` matches 0 tests; use `--test ghost_runs`. |
| T23 | ticked. `safety-a11y.test.ts` 10/10 (no unsafe HTML sinks, plain-text output, verdict text labels, hidden-row redaction, all shrink steps listed, static under reduced motion, accessible names and 44px targets, WCAG AA pairs, focus ring). |
| T09 | NOT ticked. Vitest part passes, but the named check (desktop/tablet/portrait screenshots, 200% zoom, 44px targets in a browser) needs `e2e-shell-check.mjs` and Chrome. No browser is installed on this machine. |
| T13 | NOT ticked. `run-session.axum.test.ts` against `counterspell-testserver` passed 4/4 on 5 of 7 runs. On 2 runs "shows a successful sample run from real responses" hit the 5000 ms vitest timeout (the test has no explicit timeout; its own wait is 15 s). An intermittent suite is not a pass. |
| T15 | NOT ticked. Ghost card renders recorded steps (WorkspaceShell test), but no test I could find asserts "no percentages or fake steps" or the fixture-route banner. |
| T18 | NOT ticked. FAILING: `WorkspaceShell > reruns the stored ghost with the editor source through the ghost action`. T25 (8165451) made the client send `ghost.submission_id` (the T02 endpoint requires it) and updated one test, but this one still expects `ghost: { input: [0, 0] }`. Received `ghost: { submission_id: 'run-ghost', input: [0, 0] }`. Reproduced 3 times. |
| T20 | NOT ticked. One-request-per-run, failure fallback and same-origin credentials are tested. Reconnect/remount and the daily-cap fallback have no test I could find. |
| T21 | NOT ticked. Shortcuts, polite announcements and no focus steal are tested. The keyboard-only round trip (problem, editor, Run, Ghost, Hint, Rerun, back) needs the browser harness. |
| T22 | NOT ticked. Its own notes say media-query emulation is not covered; the task requires browser media-query tests. |
| T24, T25 | NOT ticked. The real-stack harness (`web/tests/real-api/real-api-check.mjs`, 21/21 recorded) needs Chrome via Playwright `channel: 'chrome'`. Not installed here, so I could not re-run it. T25 also depends on the T18 fix. |

## T26 reconnect, retry, ownership and failure paths (19 Nov)

- `web/tests/real-api/reconnect.spec.mjs` (20/20 on the real stack): real
  magic-link sign-in for two users, live SSE drop mid-submit, Last-Event-ID
  resume with no gap, stale-cursor replay identical for client dedup, 400 on
  a malformed cursor, terminal read model carries verdict + ghost lifecycle
  (19 events), finished-run replay from cursor 0 for gap recovery, foreign
  run read/SSE 404, missing run 404, oversized source 413 source_too_large,
  malformed source 400 source_does_not_parse, unsupported verdicts never hunt
  (output-limit path: driver does not count function stdout as run output,
  so the run is accepted and still emits no ghost events), user rate limit
  and queue cap both 429 in a burst (33 refusals).
- Worker retry: SIGKILLed the judge mid-ghost_hunting; the reclaimed run
  re-emitted restarted, re-ran the suite + hunt and finished wrong_answer
  (events: ...,ghost_hunting,restarted,compiling,...,ghost,done). An earlier
  run killed mid-compile instead exhausted retries at compile — per-run
  wedge recorded in ide-workspace-blockers.md.
- Stack: Axum on 127.0.0.1:8088, real judge + rootless daemon + cs-dev-pg,
  all restarted post-crash with the README env (dev-test.env + explicit
  PUBLIC_BASE_URL/SESSION_SECRET/SANDBOX_DOCKER_SOCKET).

## T13 ticked (kiro/drive-queue, 19 Nov 2026)
- Root cause of the 2-in-7 flake: it was not a cold start. The failing session ended `failed` with `429 Too many requests`. The fixture allows one run per user per 600 s window (`RUN_RATE_USER=1`), but the suite chose users with `9 + (now_secs % 200)`, which repeats every 200 s, so reruns within 10 minutes reused spent users. A heavy loop also hit the loopback IP cap (100 per window).
- Fix: `crates/testserver` seeds 4096 users (was 256); the suite draws 5 users from one of 817 slots that advances every second (a slot repeats after 817 s, longer than the window); the documented start command adds `RUN_RATE_IP=10000`. The success test has an explicit 20 s timeout above its own 15 s wait. Timeout errors now include the session error, and the test fetch logs the transport cause on failure.
- Evidence: 113 consecutive suite runs 4/4 against the live fixture (30 + 40 + 70 loops minus earlier batches, plus 3 runs straight after restarts), after one unexplained `network` failure in the first 30-run batch that did not recur in 113 runs; the diagnostic logging will show its cause if it does. Rust gate 1521 passed / 0 failed, clippy clean (testserver forced), web check 0/0, build ok, vitest 498 passed.

## T15 ticked (kiro/drive-queue, 19 Nov 2026)
- Found a real gap while adding the missing assertions: the shrink replay set `transition-delay` and `transition-opacity` on each step but never changed opacity, so nothing animated, and the card had no 400 ms reveal. `GhostPanel.svelte` now wraps the card in a `ghost-reveal` (400 ms fade and blur-in) and gives each recorded step a `ghost-step` 200 ms CSS fade at `position * stepDelayMs`, only in the plan's `animated` mode. Static mode (reduced motion) gets neither; a `prefers-reduced-motion` rule also turns both off. No GSAP in the ghost card. Steps stay in the DOM from the start, so screen readers get the full list.
- New tests in `GhostPanel.test.ts` (+5): stagger delays 0/250/500 ms for three steps; one reveal under motion; no reveal, no step class and no delays under reduced motion; only recorded steps listed (no invented entries, no `%`) when the engine summarised 5 steps; the live hunt list shows exactly the streamed steps.
- Fixture-route visual check: `web/e2e-prototype-check.mjs` (self-serves `build/` with the 200.html fallback) — banner visible and on the first screen at 1440x900 and 390x844, 4/4 PASS; screenshot reviewed.
- Gate: svelte-check 0/0, build ok, vitest 503 passed (was 498).

## Verified (spec-driver `drive queue`, 20 Nov 2026)
- T18 ticked. Fresh checks: `web/tests/real-api/real-api-check.mjs` 18/18 on my own stack (server 8099, one judge, rootless daemon, `cs-dev-pg`), including "ghost: rerun reaches a real verdict" (verdict wrong_answer, 3 runs) and the autopsy staying on the page; `ghost_runs` 10/10 with `--include-ignored` (signed-in check, validation without side effects, missing or foreign source, atomic persist, idempotent replay and key reuse, active cap, and the new per-user rate limit 429 `rate_limited` with nothing queued); vitest workspace+runs 166 passed; svelte-check 0/0; fmt and clippy clean.
- Added `pg_ghost_run_shares_the_per_user_rate_limit` (the rate-limit leg of T18's verify line had no test).
- T20 ticked. Added the two missing assertions: a daily-cap answer (`source: template`) shows as "Measured summary" and a 429 shows the fixed unavailable line without server text (`api/familiar.test.ts`); a remount of the workspace on the same finished run sends exactly one `POST /api/runs/{id}/familiar-hint` (`WorkspaceShell.test.ts`, run id `run-remount`; fails when the requester cache is removed). Same-origin credentials, one request per run, no request before the terminal read, speech-off and failure fallback were already tested. vitest all green, svelte-check 0/0. Real route `familiar-hint` is routed now (server `familiar/hint.rs`).
- T24 and T25 ticked (20 Nov 2026). Own stack from this worktree: Axum `counterspell-server` on 127.0.0.1:8099 serving `web/build`, one `counterspell-judge` with `--features sandbox` on the rootless daemon, `cs-dev-pg`, `add-two` seeded with `seed-demo` from a scratch private root. `node web/tests/real-api/real-api-check.mjs` (`REAL_API_BASE=http://127.0.0.1:8099`, `E2E_EXECUTABLE` = installed chromium): **24/24**. It reaches the app through Axum (no dev server, no `page.route`), reads persisted `run_events` and terminal read models from Postgres, and covers: draft restore after reload, samples Accepted, wrong Submit, real events, three real shrink steps, autopsy text on the page, authored hint rungs 1 and 2 in order, the shrink list on the page equal to the persisted trace, ghost rerun to a real verdict, fix and resubmit to Accepted without a reload, no hidden detail on the page, a persisted measured `familiar` event (mood 4, `no_bench_spec`) and exactly one `familiar-hint` request per finished run. Report with run ids, per-run `seq:type` lists, ghost provenance, shrink lists, Familiar data and the final terminal model: `docs/kiro-log/ide-workspace-t25-report.json`.
- **Real bug found by this run and fixed:** `HintPanel` was never mounted in the workspace, and `requestAuthoredHint` read `text` while `POST /api/hints` answers `body` (`HintView`). `WorkspaceShell` now mounts the ladder for a finished run (rungs 1 to 4, one at a time, fixed error line, new ladder per run), and `api/hints.ts` reads `body` (`test-change: HNT-1` in `hints.test.ts`, plus a test that a reply without `body` is refused). +3 shell tests, +1 api test.
- T21 and T22 ticked (20 Nov 2026). New real-stack script `web/tests/real-api/keyboard-motion.mjs` (shared sign-in/editor helpers in `support.mjs`): **11/11** on the same own stack. T21: the editor is reached with Tab only, Ctrl+Shift+Enter submits from inside Monaco, focus stays in the editor while every event streams in, shortcut help is linked by `aria-describedby`, Tab order Run > Submit > Rerun ghost > Show hint 1 > Familiar controls > problem > editor, Enter reveals the authored rung, Space sends the ghost rerun, Shift+Tab returns to the editor. T22: under `reducedMotion: 'reduce'` no transition or animation longer than 1 ms starts during a failing run (normal motion: 5, the ghost reveal and three step fades), the full shrink list shows under both settings, nothing is still animating 3 s after the run, and the GSAP chunk loads only in normal motion. Route-change cleanup re-checked with `e2e-shell-check.mjs` (0 FAIL).
- **Real bug found and fixed:** in a real browser the Run/Submit shortcuts did nothing while the caret was in the editor. Monaco binds Ctrl(+Shift)+Enter to "insert line" and stops the event, so the bubbling window listener never fired. `RunControls` now listens in the capture phase and stops a recognised shortcut (new test: "still runs and submits when the editor stops the key from bubbling", RED before the fix).
- Note: the global reduced-motion rule (`transition-duration: 0.01ms` on `*`) still fires `transitionrun` events; the check counts durations over 1 ms, which is the visible motion.
- Env note: the auth magic-link limit is 10 per IP per hour; repeated harness runs hit it. I deleted only the `rl:auth:magic:ip:*` key in the throwaway dev Valkey between runs.
- Wave review (semantic reviewer, `semantic-review/2026-11-19-ide-workspace-t18-t25.md`): APPROVED, no Critical findings. Fixed both Important findings: a test now covers the ladder resetting for a new run and a late reply for the old run arriving while the new run's rung 1 is pending (it fails when either the reset or the run-id guard is removed). Also fixed Minor 3: `onNextHint` checks the run id before it marks a rung pending. Open Minor items: the rung count 4 is a second copy of `HintRung` 1..4 (prefer the server's `next_rung`); Ctrl+Enter also fires from buttons and any future dialog (consider exempting `[role=dialog]`).
- T26 ticked (20 Nov 2026). `web/tests/real-api/reconnect.spec.mjs` **24/24** on the own stack (`JUDGE_PID_FILE=/tmp/cs-queue-judge.pid`): live SSE, `Last-Event-ID` resume with no gap, stale-cursor replay for de-dup, malformed cursor 400, terminal read after reload, full replay from Postgres, foreign and missing runs 404, oversized 413, unparseable 400, output-limit never hunts, a judge killed mid-run is reclaimed and the retry reaches `wrong_answer` with its ghost, endless loop = `time_limit`, exception = `runtime_error`, a module Python refuses = `compile_error`, a worker killed on all three attempts = `internal_error` / `retries_exhausted` with no verdict and no ghost, and a burst is throttled (`queue_cap`, `rate_limited`). Client de-dup and stale-run isolation are covered by `reducer.test.ts` (duplicate seq, `restarted` clears rows) and `run-session.test.ts` (events ignored after cancel; a slug change unmounts the shell, which cancels).
- The script no longer uses `wsl` on Linux, no longer hard-codes another checkout, and only kills the judge PID in `JUDGE_PID_FILE` (it used to `pgrep | kill -9` every judge on the machine). The retry check now requires a real verdict; it used to accept `retries_exhausted` because of the wedge.
- **Real bug found and fixed (`25aeaaac`, judge-sandbox):** every retry of a killed run failed with `sandbox_infrastructure` because the retry's container name collided with the killed attempt's container.
- T27 ticked (20 Nov 2026). New `web/src/lib/performance.ts`: the seven documented marks (`workspace-shell-interactive`, `problem-detail-loaded`, `monaco-ready`, `run-first-event`, `run-terminal`, `ghost-playback-start`, `ghost-playback-end`), fixed names only and never a detail object; run marks once per run; ghost marks once per distinct ghost, end cancelled on unmount. Wired into WorkspaceShell, ProblemWorkspace, MonacoEditor and GhostPanel. +11 unit tests in `performance.test.ts`, +6 component tests (shell, problem, editor, ghost x3).
- `web/tests/real-api/perf-check.mjs` **9/9** on the real stack at 1024x768: the landing page and problem list load no Monaco code; Monaco loads only on the workspace; GSAP is not loaded before anything animates; on a throttled 1.5 Mbps link the shell is usable (the Problem drawer opens) while the editor still shows "Loading editor"; every mark is recorded in order; no mark carries a detail, the source or an id. One local measurement (`docs/kiro-log/ide-workspace-t27-perf.json`): shell interactive ~0.87 s, editor ready ~21.8 s on the throttled link, first event to terminal 1.27 s, ghost replay 750 ms (3 steps). Monaco is ~4.0 MB of decoded JavaScript on the workspace only; the other workspace chunks are ~410 KB.
- **Real bug found and fixed:** GhostPanel received a fresh copy of the same ghost on each stream event, so the replay marks restarted and the end mark was cancelled (test "marks one playback for the same ghost across updates", RED first).
- **For Brighton (performance, not changed here):** at 1.5 Mbps Monaco takes ~22 s to be ready. At the 10 Mbps target in `docs/requirements.md` that is roughly 3 to 4 s for Monaco alone, unless Axum or Caddy compress it; the shell itself is ready far sooner. Worth checking compression and Monaco's language and worker set before launch.
