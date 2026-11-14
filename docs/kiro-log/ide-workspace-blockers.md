# ide-workspace blockers (implementation run, 13 Nov 2026)

Tasks left unchecked in `.kiro/specs/ide-workspace/tasks.md`, with the reason and what would unblock each one.
Checked tasks: T01, T03, T04, T08, T14, T16, T19. Partial (committed, unchecked): T10, T17, T20.

## Blocked by upstream specs (not implemented yet)

- **T02 custom ghost run route.** Needs a `problems` schema table and a persisted `ghosts` row to validate the problem and fetch the stored ghost input. Neither exists in `migrations/`, and grimoire-packs (4 open), judge-sandbox (56 open) and ghost persistence are not done. Unblock: grimoire-packs catalogue tables plus judge-side ghost persistence.
- **T11 SSE reducer and stream client.** The "existing run-lifecycle browser logic" is run-lifecycle T32 (pure reducer) and T33 (stream client). Both are unchecked, and no `web/src/lib/runs` code exists. Writing them here would duplicate those tasks. Unblock: complete run-lifecycle T32 and T33, then reconcile T11 against them.
- **T12 test result panels.** Depends on T11 for the live per-run model. Unblock: T11.
- **T13 connect Run and Submit.** Needs `GET /api/runs/{id}/events` (run-lifecycle T29 to T31 are unchecked; the route is not in `crates/server/src/lib.rs`), `GET /api/problems/{slug}` (grimoire-packs), and `GET /api/languages` (language-drivers, 0 of 27 done). Unblock: those routes plus T11.
- **T18 Rerun ghost.** Depends on T02 and T13. Unblock: both.
- **T20 Familiar wiring (partial).** The adapters and the one-request guard are committed (`web/src/lib/api/familiar.ts`). Calling them from the page needs T13. The Familiar routes (`PATCH /api/me/familiar`, `POST /api/runs/{id}/familiar-hint`) are not in the server yet (efficiency-familiar: 0 of 64 done).
- **T17 AI hint display (partial).** The authored ladder, adapter and inert panel are committed. The AI text with its `AI hint` label needs the hints spec's AI response contract. `POST /api/hints` is not routed (hints: 0 of 29 done).
- **T28 full verification gate.** Needs judge-sandbox, language-drivers and hints. Not run in this pass: `cargo deny check licenses`, `docker compose config`, and the real API acceptance suite.

## Needs Brighton

- **T05 fonts.** The tokens and the Tailwind font variables already exist in `web/src/app.css`. Geist and JetBrains Mono files are not vendored, and `docs/palette.md` says they must be OFL-1.1 and checked. Unblock: approve the font packages (for example `geist`, `@fontsource/jetbrains-mono`) and note them in `THIRD-PARTY-NOTICES.md`.
- **Decision: Ctrl/Cmd+Enter inside Monaco.** In Monaco, Ctrl+Enter inserts a line below. `shortcutFor` runs code on Ctrl/Cmd+Enter, which overrides that command while the editor has focus. The spec asks for no unexpected overrides. Assumed choice: keep the shortcut and keep the Run button as the always-available path. Brighton to confirm, or pick another key.
- **Decision: Familiar art.** T19 has text, badges and controls, but no mood SVG art. Efficiency-familiar owns the art. Assumed choice: leave the art to that spec.
- **Decision: GSAP.** Not added. The Familiar uses CSS only in this pass. Add GSAP when efficiency-familiar's motion work starts.
- **Decision: markdown renderer (T06, not started).** The design leaves the renderer open. Assumed choice: a small allowlisted renderer with no raw HTML, after a licence check.

## Not started in this pass (E2E or time)

- **T06, T07, T09, T15, T21, T22, T23.** Not reached in this pass. T07 needs `monaco-editor` (MIT) added as a dependency. T15 can use T14 and T16 now.
- **T24, T25, T26.** Deferred: E2E. They need the built Axum app, Postgres, the judge and the rootless sandbox. Rule 7 says no browser or localhost E2E for now.
- **T27.** Deferred: E2E. The browser performance marks need a real browser run.

## Contract assumptions to confirm

- `GET /api/languages` is a JSON array of language summaries. The design does not fix the shape.
- The `ghost` event payload is the shape in `web/src/lib/ghost/types.ts`. The server does not emit it yet, and the field names must match judge-sandbox when it does.
- `ProblemDetail.constraints` is a list of strings. The design allowed structured objects too.
- The custom ghost route (T02) is the assumed `POST /api/runs/ghost` from design open question 1.

## Attempt 2 update (13 Nov 2026, evening)

Newly done: T06 (statement panel), T07 (lazy Monaco and Crypt theme). T09 is built but left unchecked.

- **T09 unchecked: browser checks deferred (E2E).** Built: `WorkspaceShell`, `EditorPanel`, `ProblemWorkspace`, `editor-session.ts`, route `web/src/routes/problems/[slug]/`. Unit tests cover landmarks, the problem toggle, the language picker (enabled only), draft restore and save status, and the load error state. Still needed, and deferred as E2E: desktop, tablet and portrait layout checks, 200% zoom, the 44px target check, and route-change abort in a real browser.
- **T09 side effect: SPA fallback.** `web/vite.config.js` now sets `adapter({ fallback: 'index.html' })`, so a deep link to `/problems/<slug>` reaches the app. The server already falls back to `index.html`. Brighton to confirm this config change.
- **T05 remains unchecked (fonts).** Geist and JetBrains Mono files are still not vendored. The tokens in `app.css` are in place. T09 was built on the tokens, not the font files.
- **Monaco bundle.** Monaco is a separate 2.7 MB chunk. The entry and app graphs do not reach it. It loads only through the problems route, and only when the editor mounts. Confirmed from the build output.
- **Monaco worker import.** `monaco-editor/editor/editor.worker.js?worker` (Vite's worker import). The `esm/vs/...` path does not resolve because of the package `exports` map.
- **New dependency (T07).** `monaco-editor` 0.57.0 (MIT), recorded in `THIRD-PARTY-NOTICES.md`, with its transitive `marked` (MIT), `dompurify` (MPL-2.0 OR Apache-2.0, used under Apache-2.0) and `@types/trusted-types` (MIT). The lockfile also dropped unused `jsdom` and helper packages. Tests and build pass after that change.
- **Pre-existing lint error (not from this pass).** `web/src/lib/api/client.ts` lines 137–138 (T03) fail `no-useless-assignment`. `npm run lint` will fail until those initialisers are removed. The required gate (`check`, `test`, `build`) passes.
- **T10 and T17 still partial.** T10 needs `StatusAnnouncer.svelte` and the Run/Submit wiring (T13). T17 needs the AI text contract.

## Attempt 3 update (13 Nov 2026, evening)

- **T11 done (reconciliation needed).** Added `web/src/lib/runs/reducer.ts`, `stream.ts` and `api.ts` with tests. These cover the same work as run-lifecycle T32 (pure reducer) and T33 (stream client). Brighton or the run-lifecycle run should mark those two tasks as done against this commit instead of writing them again. The shared JSON client gained an optional `headers` field for `Idempotency-Key`. Lint errors in `client.ts` (`no-useless-assignment`, noted in attempt 2) are fixed.
- **T10 done, API error preservation deferred to T13.** Shortcuts, shortcut help, the polite save-status region and the byte warning are tested. The rule that the Monaco editor's Ctrl+Enter runs samples is kept as an assumed choice (see the attempt 2 decision). Brighton to confirm.
- **T14 bug fixed (playback index 0).** `planPlayback` dropped a first step numbered 0, because it started its duplicate check at 0. The engine numbers the first failing input 0 (ghost-engine design, ShrinkStep.index). The fix accepts the first received step and keeps the duplicate and out-of-order rules. The T14 tests had assumed 1-based numbering. A new test covers index 0.
- **T15 partial (left unchecked).** `GhostPanel.svelte` covers the hunting text (no percentages), the minimized wording for wrong and crash ghosts, the timeout wording, the static list under reduced motion, the no-trust-match case and no-ghost. It does not yet handle the aborted and malformed states from the verify list. Those come from the stream and the route wiring (T13), so the route can pass them in. The visual check with the prototype banner is deferred: E2E.
- **T17 done for display; AI request still waits on the hints spec.** `HintPanel` shows optional AI text with the exact label "AI hint" as plain text, and an unsafe string stays inert. Calling `POST /api/hints` and showing the server reply still needs the hints spec, which has no routes yet.

## Attempt 5 update (13 Nov 2026, night)

- **T15 partial (left unchecked).** `GhostPanel.svelte` now handles the aborted and cut-off endings (`aborted`, `cancelled`, `expired`, `internal_error`). These show "The ghost search did not finish", not "No ghost found". A malformed ghost payload (the new `ghostError` prop) shows "could not be verified" and no steps. Unit tests: 11 pass. `npm run check`, the full web test run (239 tests) and `npm run build` pass. Still open: the prototype-banner visual check, which is deferred: E2E. The route passing `ghostError` comes from T13.
- **T02 still blocked.** The run flow has no problem `input_schema` and no stored ghost lookup. The seams in `crates/server/src/seams.rs` carry only a slug (`ProblemRef`). Unblock: grimoire-packs catalogue tables and ghost persistence from judge-sandbox.
- **T05 still blocked on Brighton.** The Geist and JetBrains Mono font files need approval (see the attempt 2 decision above).
- **T09, T21, T22, T23 deferred: E2E.** Their verification lists browser layout, zoom, keyboard-only and media-query checks.
- **T13, T18, T20 blocked by upstream.** They need the run SSE route (run-lifecycle T29 to T31), the problem and language routes, and the Familiar and hint routes.
- **T24 to T27 deferred: E2E.** T28 needs `cargo deny`, `docker compose config` and the real acceptance suite, which need services not available here.

## Attempt 6 update (14 Nov 2026, non-interactive resume)

No new task was completed in this pass. Checked tasks in `tasks.md`: T01, T03, T04, T06, T07, T08, T10, T11, T12, T14, T16, T19. Partial, left unchecked: T09, T15, T17, T20.

- **Stale entries corrected.** T08 (draft persistence) and T12 (test panels) are checked in `tasks.md`. The earlier notes that called T12 blocked by T11 are out of date, because T11 is done.
- **Web gate re-run this session:** `npm run check` reports 0 errors and 0 warnings. `npx vitest run` reports 36 files and 243 tests passing. The Vitest pool warning about happy-dom is an open Minor finding.
- **T02 still blocked.** The `migrations/` folder has run-lifecycle and auth migrations only. There is no `problems` table, no `ghosts` table and no catalogue load path. Unblock: a grimoire-packs catalogue migration and ghost persistence from judge-sandbox. grimoire-packs T20 and T28 to T30 are also blocked, so this will not change until those land.
- **T13 still blocked.** `crates/server/src/lib.rs` routes only `POST /api/runs`, `GET /api/runs/{id}`, cancel and health. `GET /api/runs/{id}/events` is run-lifecycle T29 to T31, all unchecked. `GET /api/problems/{slug}` and `GET /api/languages` do not exist.
- **T18 and T20 still blocked.** T18 needs T02 and T13. T20 needs T13 and the Familiar and hint routes, which are not in the server.
- **T05 still blocked on Brighton.** The Geist and JetBrains Mono font packages need approval and a `THIRD-PARTY-NOTICES.md` entry.
- **T09, T15 (visual check), T21, T22, T23:** the remaining checks in each task need a real browser (focus order, zoom, media-query behaviour, contrast). Rule 7 defers them as E2E. The unit parts that exist are already covered by the component tests listed in the attempt 2 to 5 notes.
- **T24 to T27:** deferred: E2E.
- **T28:** blocked on judge-sandbox (0 of 56 done), language-drivers (0 of 27) and hints (0 of 29), and it needs services not available here.
- **Brighton questions unchanged:** the Ctrl/Cmd+Enter override inside Monaco, the `adapter({ fallback: 'index.html' })` change in `web/vite.config.js`, and the font approval.
