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
