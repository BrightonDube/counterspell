# Tasks: ide-workspace

All tasks are implementation tasks for `requirements.md` and `design.md`. This spec-generation run does not execute any task. Each task is sized for 30 minutes or less. `[P]` means it can run in parallel with other tasks in its wave after its dependencies are complete. `[S]` means sequential or dependency-bound. `[UI]` marks a UI-only task so work implemented outside Kiro can be separated cleanly.

## Conventions for every task

- Preserve the six-package workspace and canonical paths from `repo-skeleton`. Do not add React, Next.js, Pyodide, a second frontend, a browser judge, or a new Cargo package.
- Follow `.kiro/steering/frontend.md`, `api-conventions.md`, `security.md`, `structure.md`, `tech.md`, and the relevant existing `run-lifecycle`, `ghost-engine`, `sandbox`, and `rust-teaching` rules.
- Production workspace requests use the actual Axum API and authenticated same-origin cookies. Fixtures are allowed only under the explicit prototype route and must display a persistent `Prototype fixture — not a real run` banner.
- Never render user output, compiler output, ghost values, AI text, or other untrusted text as HTML. Never log source, hidden data, cookies, prompts, or secrets.
- Run-lifecycle event names and payload visibility remain authoritative. Hidden tests expose ordinal, verdict and time only. `Last-Event-ID` and persisted `seq` are not optional.
- Rust production code has no `unwrap()` or `expect()` outside tests, returns contextual errors, and ends the task with a 3–5 line Rust lesson appended to `docs/kiro-log/rust-lessons.md` in the existing format. UI-only tasks need no Rust lesson.
- New dependencies require a licence check and an update to `THIRD-PARTY-NOTICES.md`. Stop and ask Brighton if `cargo deny` reports an unapproved licence.
- Verify relevant work with `npm run check`, `npm test` where available, `npm run build`, and focused Cargo checks. Do not claim the full flow is real until the integration tasks use the built Axum app and sandbox-backed judge.

## Wave 0: contracts and safe seams

- [x] **T01 [P] (25 min) Freeze browser API and view-model contracts**
  - Files: `web/src/lib/api/types.ts`, `web/src/lib/runs/types.ts`, `web/src/lib/ghost/types.ts`, `web/src/lib/familiar/types.ts`, focused tests.
  - Define strict browser types for `ProblemDetail`, `LanguageSummary`, run read models, public SSE envelopes, sample/hidden test rows, ghost/autopsy payloads, authored hints, measured Familiar data, and stable API errors. Keep raw server data at the adapter boundary and expose only public view models to components.
  - Add schema guards for hidden test payloads, bounded text, shrink trace entries, terminal results, and measured-only Familiar fields. Fixtures belong only in test files, not production modules.
  - Depends on: none.
  - Verify: unit tests reject a hidden test carrying input/expected, reject missing ghost provenance, reject unbounded output, and accept documented public shapes; `cd web && npm run check`.
  - Preserve: exact event names from `api-conventions.md`; no raw JSON-to-UI escape hatch.

- [ ] **T02 [P] (30 min) Add the custom ghost-run Axum seam**
  - Files: `crates/server/src/runs/custom.rs` or the existing run module, `crates/server/src/lib.rs`, `crates/common/` only if a shared request type is needed, API tests.
  - Implement `POST /api/runs/ghost` as designed: authenticated ownership check for the source submission, problem/schema validation, language/source validation, queue-cap/rate-limit checks, and atomic enqueue through the same Postgres job path. The request contains current source and a stored ghost reference/input; the server never executes it.
  - Keep the judge responsible for comparator and verdict. Return `202 {run_id,status:"queued",mode:"run",custom:true}` so the existing run state machine remains compatible, and use stable JSON errors. Do not reveal hidden data or bypass run leases.
  - Depends on: `run-lifecycle` enqueue and ownership seams.
  - Verify: ownership, invalid schema, source limit, idempotency/retry, queue cap, and atomic insert tests; `cargo test -p counterspell-server custom`.
  - Preserve: server has no Bollard dependency or Docker socket; Postgres remains the queue and source of truth.
  - Rust lesson: explain why the route validates and enqueues but never executes source code, and how `?` preserves database error context.

- [x] **T03 [P] (25 min) Wire problem and language API adapters**
  - Files: `web/src/lib/api/client.ts`, `web/src/lib/api/problems.ts`, `web/src/lib/api/languages.ts`, adapter tests.
  - Implement same-origin `GET /api/problems/{slug}` and `GET /api/languages` loaders with abort support, stable error decoding, safe markdown input, and strict response validation. Do not put production problem content in the bundle.
  - Depends on: T01 and the catalogue/language API seams.
  - Verify: loading, 401, 404, malformed JSON, timeout, and abort tests; no secrets or raw server error internals are exposed.
  - Preserve: enabled language flags from the server are authoritative.

- [x] **T04 [P] (25 min) Add the fixture-only prototype route boundary**
  - Files: `web/src/routes/prototype/ide-workspace/+page.svelte`, fixture module under `web/src/lib/prototype/`, route tests.
  - Create a visually useful shell fixture for UI work that has a persistent, high-contrast `Prototype fixture — not a real run` banner. Use unmistakable fixture ids and prevent fixture actions from calling production run endpoints.
  - Depends on: T01.
  - Verify: production `/problems/[slug]` has no fixture fallback; prototype route is visibly labelled and cannot be mistaken for judge output.
  - Preserve: fixture data may show design states only; it must never claim real execution, shrinking, timing, or acceptance.

## Wave 1: UI foundation and entry

- [ ] **T05 [UI] [P] (25 min) Establish the workspace Steel Night shell and semantic tokens**
  - Files: `web/src/app.css`, shared UI primitives, route skeleton, font loading configuration.
  - Apply `docs/palette.md` semantic variables, Geist/JetBrains Mono, dark launch theme, focus token, status tokens, and calm panel surfaces. Do not put raw hex values in components; keep documented Monaco values in the theme boundary.
  - Depends on: T04 and the existing frontend scaffold.
  - Verify: `npm run check && npm run build`; inspect that success green and danger red are reserved correctly and no purple/ambient animation is introduced.
  - Preserve: no second styling system and no landing-page animation in the IDE.

- [x] **T06 [UI] [P] (25 min) Build the problem statement panel**
  - Files: `web/src/lib/workspace/ProblemPanel.svelte`, safe statement renderer, component tests.
  - Show title, breadcrumb/track, concepts, difficulty, constraints, signature, examples, and collapsible detail sections. Render statement markdown safely with raw HTML disabled or allowlisted.
  - Depends on: T01, T03, T05.
  - Verify: keyboard expand/collapse, safe markup, missing optional fields, long constraints, and sample rendering tests.
  - Preserve: sample values are data, not HTML.

- [x] **T07 [UI] [P] (25 min) Add the lazy Monaco editor and Crypt theme**
  - Files: `web/src/lib/editor/MonacoEditor.svelte`, `web/src/lib/editor/crypt-theme.ts`, Monaco loader tests.
  - Dynamically import Monaco on workspace entry/editor focus, define the documented `crypt` theme, configure Python/JavaScript language modes, expose an accessible editor label, and dispose the model/editor on route leave.
  - Depends on: T05 and T03.
  - Verify: editor is not in the initial shell bundle, theme tokens match `docs/palette.md`, language switching preserves models, and dispose is called on unmount.
  - Preserve: no browser execution, Pyodide, or fabricated diagnostics.

- [x] **T08 [UI] [P] (25 min) Implement per-problem/language draft persistence**
  - Files: `web/src/lib/editor/draft-store.ts`, `source-validation.ts`, tests.
  - Store source under a versioned `(problem slug, language)` key with debounced writes, restore valid drafts, count UTF-8 bytes, and preserve the in-memory value if storage is unavailable. Do not store runs, hidden values, AI prompts, cookies, or secrets.
  - Depends on: T01.
  - Verify: language isolation, reload restoration, multibyte byte counts, storage quota failure, corrupted entry recovery, and source copy fallback tests.
  - Preserve: client checks are advisory; server VAL-1–VAL-3 remain authoritative.

- [ ] **T09 [UI] [S] (30 min) Compose the responsive WorkspaceShell**
  - Files: `web/src/routes/problems/[slug]/+page.svelte`, `web/src/lib/workspace/WorkspaceShell.svelte`, `EditorPanel.svelte`, responsive CSS.
  - Compose problem, editor, controls, tests, ghost/autopsy, hints, and Familiar regions. Implement wide desktop columns, tablet drawers/tabs, narrow single-column order, collapse state, landmarks, and focus return.
  - Depends on: T05, T06, T07, T08.
  - Verify: desktop/tablet/portrait screenshots or browser checks, 200% zoom, no core horizontal overflow, 44px primary targets, route-change aborts, and no old slug updates.
  - Preserve: Familiar never covers editor text; shell stays useful while heavy assets load.

## Wave 2: editing and run lifecycle UI

- [x] **T10 [UI] [P] (25 min) Add Run/Submit controls and draft readiness rules**
  - Files: `web/src/lib/workspace/RunControls.svelte`, `keyboard.ts`, `StatusAnnouncer.svelte`, component tests.
  - Add Run, Submit, language selection, byte warning, save state, shortcut help, disabled/loading rules, and accessible status text. Use `Ctrl/Cmd+Enter` for Run and `Ctrl/Cmd+Shift+Enter` for Submit without overriding Monaco editing commands unexpectedly.
  - Depends on: T08 and T09.
  - Verify: controls remain safe during duplicate clicks, source-too-large warning reports bytes, keyboard shortcuts, visible focus, and API error preservation tests.
  - Preserve: controls never infer a verdict from local validation.

- [x] **T11 [UI] [P] (30 min) Integrate the persisted SSE reducer and stream client**
  - Files: `web/src/lib/runs/reducer.ts`, `stream.ts`, `api/runs.ts`, tests or reconciliation with existing `run-lifecycle` browser logic.
  - Use one idempotency key per user action, connect to real SSE, apply `seq` ordering, ignore duplicates/stale events, detect gaps, reconnect using the last sequence, handle `204`, close after terminal, and fetch the authoritative run read model.
  - Depends on: T01 and existing `run-lifecycle` T32/T33 or their equivalent.
  - Verify: fake EventSource unit tests for duplicates, gaps, reconnects, `restarted`, terminal close, malformed events, and 429 errors; no in-memory event source is used as production truth.
  - Preserve: public payload visibility and ownership rules remain server-side.

- [ ] **T12 [UI] [S] (25 min) Build sample and hidden test result panels**
  - Files: `web/src/lib/workspace/TestPanel.svelte`, test row components, tests.
  - Render queued/claimed/compiling/running/terminal statuses, sample input/expected/got/time, and hidden ordinal/verdict/time rows. Add text/icons for every status and safe truncated plain-text output.
  - Depends on: T10, T11.
  - Verify: sample detail is visible, hidden detail is structurally impossible from the view model, output containing `<script>` stays text, compile/runtime/TLE/MLE/OLE/internal states are distinct, and `restarted` clears transient rows.
  - Preserve: never display hidden input, expected output, or hidden stdout/stderr.

- [ ] **T13 [S] (25 min) Connect Run and Submit to the production route**
  - Files: workspace page orchestration, `web/src/lib/api/runs.ts`, integration tests.
  - Wire current Monaco source to `POST /api/runs`, set the active run id, stream events, preserve the draft, keep the prior completed run visible until the replacement has a terminal result, and handle navigation cancellation.
  - Depends on: T09, T11, T12, and the real run-lifecycle API.
  - Verify: against an Axum test server with real JSON envelopes, a successful sample Run and a Submit error are shown from responses rather than mocked success; validation/rate-limit/ownership errors preserve source.
  - Preserve: this task may use a scripted judge only for isolated API tests; full success proof is T25.

## Wave 3: ghost, autopsy, hints, and Familiar

- [x] **T14 [UI] [P] (25 min) Implement the recorded shrink playback controller**
  - Files: `web/src/lib/ghost/shrink-playback.ts`, tests.
  - Build a pure controller that accepts only received ordered steps, preserves head/tail omission metadata, verifies final input agreement, caps total display time near 3 seconds, and selects static mode under reduced motion.
  - Depends on: T01 and T11.
  - Verify: no invented values, duplicate/out-of-order handling, final-step equality, omitted-step rendering, deterministic timing decisions, and reduced-motion static-list tests.
  - Preserve: the controller cannot manufacture intermediate inputs or outputs.

- [ ] **T15 [UI] [S] (30 min) Build Ghost Hunting and Ghost card**
  - Files: `web/src/lib/workspace/GhostPanel.svelte`, `GhostHunting.svelte`, `ShrinkTrace.svelte`, animation boundary.
  - Transition only on server `ghost_hunting`, show honest indeterminate search text, replay actual `shrink_step` values, show final minimized-counterexample wording, and handle no-ghost, aborted, malformed, timeout, and reduced-motion states.
  - Use one 400ms calm reveal and 150–250ms per-step CSS crossfades only when motion is allowed. Keep GSAP out of the ghost card; GSAP belongs to the Familiar boundary.
  - Depends on: T12, T14.
  - Verify: fixture route visual test has the prototype banner; production component tests prove no percentages or fake steps appear; screen-reader list contains all actual steps.
  - Preserve: a timeout ghost is shown according to engine metadata and is not falsely called minimized.

- [x] **T16 [UI] [S] (30 min) Build the Bug Autopsy card**
  - Files: `web/src/lib/workspace/AutopsyCard.svelte`, `web/src/lib/ghost/autopsy.ts`, tests.
  - Show failing input, expected, actual/crash/timeout, failure kind, engine explanation, structural differences, truncation marker, concept tags, and Authored explanation or AI hint label. Use success/danger semantic presentation plus text/icons.
  - Depends on: T01, T15.
  - Verify: expected/actual/difference paths are text-only, incomplete payloads produce unavailable state, authored versus AI labels are exact, timeout wording does not claim shrinking, and hidden canaries never render.
  - Preserve: the browser does not recompute or contradict the trusted engine diff.

- [ ] **T17 [UI] [P] (25 min) Add authored hint ladder and AI hint display**
  - Files: `web/src/lib/workspace/HintPanel.svelte`, `web/src/lib/api/hints.ts`, tests.
  - Reveal authored rungs in order, preserve already revealed text, show retry/cap states, and display optional server-provided AI text with an explicit `AI hint` label. Render all hint text as plain text.
  - Depends on: T01, T16.
  - Verify: out-of-order requests do not reveal later rungs, failed requests do not fabricate text, cap fallback is honest, and unsafe hint strings remain inert.
  - Preserve: no full solution is generated client-side.

- [ ] **T18 [UI] [S] (30 min) Wire Rerun ghost as a real custom run**
  - Files: `web/src/lib/workspace/RunControls.svelte`, `web/src/lib/api/runs.ts`, custom result integration tests.
  - Add a labelled `Rerun ghost` action that sends the current source and stored ghost through `POST /api/runs/ghost`, creates a new run id, and displays one public custom-ghost test. Keep the failed run/autopsy intact.
  - Depends on: T02, T13, T16.
  - Verify: ownership, schema, rate limit, idempotency, real queue event, one public custom result, and no unrelated hidden detail; API errors do not erase the autopsy.
  - Preserve: the source runs only in the trusted sandbox.

- [x] **T19 [UI] [P] (30 min) Build measured Familiar component and state reducer**
  - Files: `web/src/lib/familiar/Familiar.svelte`, `familiar-state.ts`, `how-it-judges.svelte`, SVG mood assets, tests.
  - Implement five measured moods, dock/hide/mute controls, text equivalent, the “How does it judge me?” popover, and semantic visual rules. Consume only `complexity`, `familiar`, or terminal measured data; code edits never change mood.
  - Use lazy-loaded SVG art, calm 120–200ms transitions, and no live typing guess. Keep the Familiar inside the result region and never over editor text.
  - Depends on: T01, T09, T11.
  - Verify: measured-only state tests, no-bench correctness state, low-confidence observational copy, hide/mute preference state, aria-live text, and reduced-motion static badge.
  - Preserve: never use “optimal”; top condition says “Matches reference complexity”.

- [ ] **T20 [S] (25 min) Wire Familiar preferences and one speech request per run**
  - Files: `web/src/lib/api/familiar.ts`, workspace orchestration, Familiar tests.
  - Persist mute/hide through `PATCH /api/me/familiar`, request `POST /api/runs/{id}/familiar-hint` once after terminal data is fetched, guard remount/reconnect duplicates, and display returned AI/template fallback without changing verdict or mood.
  - Depends on: T19 and the Familiar API seam.
  - Verify: one request across reconnect/remount, failure/timeout/cap fallback, no request before terminal result, same-origin credentials, and no prompt/code logging.
  - Preserve: speech is words only; measured data remains authoritative.

## Wave 4: accessibility, resilience, and real integration proof

- [ ] **T21 [UI] [P] (25 min) Add keyboard navigation and focus management**
  - Files: `web/src/lib/workspace/keyboard.ts`, drawers/tabs/popovers, `StatusAnnouncer.svelte`, tests.
  - Implement landmarks, tab order, shortcuts, Escape close/focus return, editor/results focus, hint/ghost actions, and throttled polite announcements for meaningful phase changes.
  - Depends on: T09, T10, T12, T15, T17, T19.
  - Verify: keyboard-only path from problem to editor to Run/Submit to Ghost/Autopsy/Hint/Rerun and back to editor; async events never steal focus; shortcut help is discoverable.
  - Preserve: Monaco's own editing commands remain usable.

- [ ] **T22 [UI] [P] (25 min) Add reduced-motion and animation lifecycle tests**
  - Files: animation boundary components, `shrink-playback.ts`, Familiar, tests.
  - Honour `prefers-reduced-motion`, disable CSS transitions and Familiar GSAP timelines/bursts/shakes/blur transitions, retain static trace and status badges, and dispose lazy animation resources on unmount and route change.
  - Depends on: T14, T15, T19.
  - Verify: browser media-query tests assert no timeline is created under reduced motion; ordinary mode plays once; no ambient loop exists; route changes do not leave timers or timelines.
  - Preserve: reduced motion removes motion, not information.

- [ ] **T23 [UI] [P] (30 min) Add output safety, accessibility, and contrast acceptance tests**
  - Files: focused component tests and browser test support under `web/tests/`.
  - Cover `<script>`/HTML output, hidden canary absence, accessible names, landmarks, focus rings, ordered shrink list, aria-live Familiar, status text/icons, keyboard operations, 200% zoom layout, 44px targets, and palette contrast targets.
  - Depends on: T12, T16, T19, T21, and the chosen test tooling.
  - Verify: tests fail if output uses unsafe HTML, a status relies only on colour, or a reduced-motion user loses trace information.
  - Preserve: do not weaken tests to accommodate visual shortcuts.

- [ ] **T24 [S] (30 min) Add real Axum browser harness and test data path**
  - Files: `web/tests/real-api/` or the agreed browser harness, `docker-compose.yml`/CI only if required for test services, test-pack wiring.
  - Start or connect to the built SvelteKit output served by Axum with authenticated test user, Postgres, trusted judge, rootless sandbox, and a small original pack. Do not intercept successful run responses. If Playwright is used, add it only after a licence check and document browser installation.
  - Depends on: T02, T03, T13, T18, T20, and upstream queue/sandbox/pack specs.
  - Verify: the harness reaches the app through Axum, not a frontend dev server, and can observe persisted events and terminal reads.
  - Preserve: any local fixture route is excluded from this harness.

- [ ] **T25 [S] (30 min) Prove the judge journey without reload**
  - Files: `web/tests/real-api/ide-workspace.spec.*`, test support only.
  - Against the actual API/judge stack: open a published problem, restore drafts, Run samples, Submit a deliberately wrong solution, observe real events, watch real shrink steps, read the real Bug Autopsy, reveal hints, rerun the ghost, fix the source, resubmit, reach Accepted, and observe measured Familiar plus one speech request.
  - Assert no hidden canary appears, no result is fabricated, and the browser never reloads or requires manual intervention between failure and success.
  - Depends on: T24.
  - Verify: record event ids, run ids, terminal read model, ghost provenance, shrink trace equality, and final Familiar data in the test report.
  - Preserve: a passing test must be caused by the sandbox-backed judge, not a mocked success response.

- [ ] **T26 [S] (30 min) Prove reconnect, retry, ownership, and failure paths**
  - Files: `web/tests/real-api/reconnect.spec.*`, server integration tests if required.
  - Drop the SSE connection at multiple sequence positions, resume with `Last-Event-ID`, assert de-duplication and gap recovery, simulate worker retry/restarted, reload after terminal, and test foreign/missing runs, rate limits, source validation, internal error, timeout, compile error, and no-ghost outcomes.
  - Depends on: T24 and T25.
  - Verify: no duplicate rows, no stale run updates a new problem, no hidden data leaks, and an infrastructure failure is never shown as Wrong Answer.
  - Preserve: the stream is replayed from Postgres, not a client cache.

- [ ] **T27 [S] (25 min) Measure workspace performance and lazy loading**
  - Files: browser performance test/support, `web/src/lib/performance.ts` if needed, no user data in marks.
  - Add shell/editor/first-event/terminal/ghost-playback marks, check Monaco and Familiar-GSAP lazy loading, exercise a tablet viewport, and confirm the shell stays responsive while heavy assets load.
  - Depends on: T09, T15, T19, T25.
  - Verify: marks contain no source/user identifiers; report bundle/chunk behaviour and measured times without turning estimates into promises.
  - Preserve: do not preload heavy assets on unrelated routes.

- [ ] **T28 [S] (30 min) Run the full implementation verification gate**
  - Files: fixes only if a check fails; no planned new feature files.
  - Run `cargo fmt --all -- --check`, focused and workspace Rust tests, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cd web && npm ci && npm run check && npm test && npm run build`, `cargo deny check licenses`, and `docker compose config`. Run the real API acceptance suite with required services.
  - Depends on: T25, T26, T27 and all upstream API/sandbox dependencies.
  - Verify: all checks pass; production route uses built static output through Axum; no server Docker socket; no prototype fixture is used by the real journey; changed files stay within this feature's planned surfaces and approved upstream seams.
  - Preserve: do not begin Java/C++, live typing guesses, dashboard, or other stretch work as part of this gate.

## Dependency summary

- **Wave 0:** T01, T02, T03, and T04 are independent after the existing upstream seams; T02 is server-side, T01/T03/T04 are browser/API boundary work.
- **Wave 1:** T05–T08 can run in parallel after Wave 0. T09 follows the shell/editor/statement pieces.
- **Wave 2:** T10 and T11 can run in parallel after the editor/contracts; T12 follows T11; T13 follows T09, T11, and T12.
- **Wave 3:** T14 and T17/T19 can run in parallel where their shared contracts are ready. T15 follows T14/T12, T16 follows T15, T18 follows T02/T13/T16, and T20 follows T19.
- **Wave 4:** T21–T23 are independent UI quality tasks once their components exist. T24 is the real harness gate; T25 and T26 are sequential real-flow proofs; T27 can run after the main route works; T28 is last.

## Completion checklist

- [ ] A published problem loads from Axum with statement, constraints, signature, samples, starter, and only enabled languages.
- [ ] Monaco uses the Crypt theme, is lazy-loaded, accessible, and preserves separate drafts per problem/language.
- [ ] Run samples show real input/expected/got and Submit shows live hidden-test-safe events.
- [ ] SSE reconnects with `Last-Event-ID`, de-duplicates by sequence, handles gaps/retries, and fetches the authoritative terminal result.
- [ ] A real wrong submission enters Ghost Hunting, replays only real shrink steps, and shows a real minimized counterexample in Bug Autopsy.
- [ ] Authored hints are ordered; AI text is labelled; Rerun ghost is a real sandboxed custom run.
- [ ] A learner can fix and resubmit to Accepted without a page reload or manual intervention.
- [ ] Familiar mood is measured-only, speech is labelled, preferences work, and reduced motion preserves information without effects.
- [ ] Keyboard navigation, screen-reader status, focus, contrast, touch targets, output escaping, hidden-data protection, and 200% zoom are tested.
- [ ] The definition-of-done path uses the built Axum app and actual judge/sandbox, never mocked success responses or prototype fixtures.
- [ ] Full formatting, lint, tests, frontend build, licence, Compose, and real-flow verification pass.
