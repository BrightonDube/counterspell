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
