# Tasks: break-room

All tasks are implementation tasks for the design above. No task in this document is being executed by this spec-generation run. Each task is sized for 30 minutes or less. `[P]` means it can run in parallel with other tasks in the same wave once its dependencies are complete. `[S]` means it is sequential or has a required dependency. Every task is `[stretch]`: nothing starts until the gate in T00 passes.

## Wave 0: gate and intake

- [ ] **T00 [S] (15 min) Confirm the freeze-safe core before any break-room work**
  - Files: none.
  - Check the release-checklist items that apply to the core: the real fail → ghost flow runs on the VPS, the Postgres queue and SSE replay work, and the auth, IDE, and dashboard specs are complete.
  - If any check fails, stop and record the decision to Brighton. Do not start T01.
  - Depends on: auth, run-lifecycle, ghost-engine, ide-workspace, dashboard-fsrs specs complete or deliberately cut.
  - Verify: a written gate note in the hand-off that lists each check as pass or fail.
  - Preserve: the rule that break room is the first feature cut.

- [ ] **T01 [S] (25 min) Licence and dependency intake for the chess packages**
  - Files: `web/package.json`, `THIRD-PARTY-NOTICES.md`.
  - Add `chess.js` (BSD-2-Clause) and `cm-chessboard` (MIT) with exact pinned versions. Read each package's `LICENSE` file from the installed copy, and record the name, version, licence and source URL in the notices.
  - Depends on: T00.
  - Verify: `npm ls chess.js cm-chessboard` shows both pinned; the notices list both with the licences read from their files; no other chess package appears in `web/package-lock.json`.
  - Preserve: licence rules. Refuse any GPL or AGPL package (Stockfish, chessground) and report it.

- [ ] **T02 [P] (20 min) Confirm the Lichess CSV licence and column names before import work**
  - Files: `deploy/breakroom/README.md` (section "Source and licence" only).
  - Record the CC0 licence statement, the source URL (`database.lichess.org`, puzzles section), the expected column header, and the rule that the CSV is decompressed locally with `zstd -d` before import.
  - Depends on: T00.
  - Verify: the README section cites the source. Column names match the design's staging table.
  - Preserve: "CC0 CSV only, no AGPL code" (R2.1, R2.2).

## Wave 1: schema, import, and the server module

- [ ] **T03 [S] (25 min) Add the break-room migration**
  - Files: `migrations/<timestamp>_breakroom.sql` (created with `sqlx migrate add breakroom`).
  - Create `puzzles` with its rating index, and `game_scores` with the cascade foreign key, the game check, the `octet_length(meta::text)` check, the user and game index, and the partial unique index `game_scores_one_puzzle_per_day`. Use the exact SQL in the design.
  - Depends on: the auth migration (for `users`).
  - Verify: `sqlx migrate run` on a throwaway database succeeds. `\d game_scores` shows the cascade and the partial index. Inserting a second `chess_puzzle` row for the same user and date fails.
  - Preserve: additive migration only. No change to existing tables.

- [ ] **T04 [P] (30 min) Write the one-time puzzle import script and its synthetic fixture**
  - Files: `deploy/breakroom/import-puzzles.sql`, `crates/server/tests/fixtures/breakroom-sample.csv`.
  - The script follows the design: staging table, `\copy` with `CSV HEADER`, filtered insert with `ON CONFLICT (id) DO NOTHING`, `source_date` from a psql variable, staging table dropped.
  - The fixture has 12 hand-written rows that are labelled synthetic in a comment line at the top of the README (not inside the CSV). Use no real Lichess rows.
  - Depends on: T03.
  - Verify: run the script against the throwaway database with the fixture (the filter should keep the rows that fit the band). Run it twice and confirm the second run inserts nothing.
  - Preserve: the import never reads from the network.

- [ ] **T05 [P] (25 min) Pure daily selection functions and unit tests**
  - Files: `crates/server/src/breakroom/daily.rs`.
  - Implement `sast_date(utc: OffsetDateTime) -> Date` (adds two hours, no timezone library) and `daily_index(days: i64, count: i64) -> Option<i64>` (`None` when `count <= 0`, otherwise `days.rem_euclid(count)`). Reuse the auth crate's `Clock` trait for the current time, not a new clock.
  - Depends on: T03 (for the crate module path only; no database needed).
  - Verify: `cargo test -p counterspell-server breakroom::daily` passes. Tests cover 23:59 and 00:01 UTC boundaries, a negative `days` value, `count == 0`, and a property test that `daily_index` stays in `0..count` for random inputs with `count >= 1`.
  - Preserve: no `unwrap()` or `expect()` outside tests.
  - Rust lesson: explain `Option` as "no value" and why `rem_euclid` is safe for negative numbers.

- [ ] **T06 [S] (30 min) Puzzle repository and the daily handler**
  - Files: `crates/server/src/breakroom/handlers.rs` (daily part), `crates/server/src/breakroom/mod.rs` (partial), `crates/server/src/breakroom/daily.rs` (repo functions).
  - Add `count_puzzles` and `puzzle_at_offset` with `sqlx` parameterised queries. The handler returns the JSON shape in the design, and `503 not_imported` when the count is zero.
  - Depends on: T04, T05.
  - Verify: integration test against the imported fixture returns the same puzzle for the same SAST date, and `not_imported` on an empty table. Ensure the response has no field beyond those in the design.
  - Preserve: public read, no sign-in required (R3.4).
  - Rust lesson: explain how a typed `Result` error turns into a JSON `not_imported` response.

- [ ] **T07 [S] (30 min) Score validation, scoring, and the score handlers**
  - Files: `crates/server/src/breakroom/scores.rs`, `crates/server/src/breakroom/handlers.rs` (score part).
  - Implement the tagged request enum with `deny_unknown_fields`, `score_for` with the table in the design, the `wrong_date` and `not_solved` checks against the SAST date, and the `meta` size check. Insert with `ON CONFLICT DO NOTHING` for puzzle scores, and read back the id. Use the `CurrentUser` extractor from auth for the user id. Use the Valkey limiter from auth for 30 per hour.
  - Depends on: T03, T05, auth spec (`AuthUser`, rate limiter).
  - Verify: unit tests for `score_for` edge values. Integration tests: 201 on first save, 200 `already_recorded` on repeat, 401 when signed out, 422 `wrong_date`, 422 `not_solved`, 400 for an unknown field, 429 after 30 saves.
  - Preserve: the client never sends a score number (R5.2). Errors are stable JSON codes.
  - Rust lesson: explain `#[serde(tag = "game")]` as a tagged enum, and why `deny_unknown_fields` protects the server.

- [ ] **T08 [S] (20 min) Own-score listing and the router mount**
  - Files: `crates/server/src/breakroom/mod.rs` (`router()`), the server router file (one line only: `.nest("/api/breakroom", breakroom::router(state))`).
  - `GET /api/breakroom/scores` returns the latest 10 rows for the session user only.
  - Depends on: T07.
  - Verify: a second user's scores never appear (integration test). An unknown `/api/breakroom/*` path returns an API error, not the SPA page (repo-skeleton rule). The server router diff is one line.
  - Preserve: R1.1 and R1.5 (one mount line, no other imports from breakroom).

- [ ] **T09 [P] (20 min) Account-deletion cascade test**
  - Files: `crates/server/tests/breakroom_cascade.rs`.
  - Insert scores for a user, delete the user through the auth deletion path, and assert the `game_scores` rows are gone.
  - Depends on: T03, T07.
  - Verify: the test passes against the throwaway database.
  - Preserve: R10.4.

## Wave 2: browser logic (pure modules first)

- [ ] **T10 [P] (25 min) Minimax module and tests**
  - Files: `web/src/lib/break/minimax.js`, `web/src/lib/break/minimax.test.js` (verification only).
  - Implement `chooseMove` as specified: negamax with alpha-beta, depth 2, material evaluation, checkmate score adjusted by ply, captures first, seeded tie-break, node budget and deadline with `timedOut` reporting.
  - Depends on: T01.
  - Verify: `vitest --run web/src/lib/break/minimax.test.js`. The bot plays the mate in one from a fixed position, returns a legal move in every position of a small fixed list, and returns the same move for the same seed.
  - Preserve: no Stockfish, no server call.

- [ ] **T11 [P] (25 min) Puzzle state machine and tests**
  - Files: `web/src/lib/break/puzzle.js`, `web/src/lib/break/puzzle.test.js` (verification only).
  - Implement `tryMove(state, uci)` over `chess.js`. Play the opponent's first move automatically. Count mistakes. Report solved and failed states.
  - Depends on: T01.
  - Verify: `vitest --run web/src/lib/break/puzzle.test.js`. Covers the correct line, a wrong legal move (counts as one mistake), an illegal move (shows a message and does not count), and the solved state.
  - Preserve: R4.1 to R4.3.

- [ ] **T12 [P] (20 min) Memory game logic and tests**
  - Files: `web/src/lib/break/memory.js`, `web/src/lib/break/memory.test.js` (verification only).
  - Implement `newRound`, `checkPress`, and `level` as specified, with an injected `rng`.
  - Depends on: none.
  - Verify: `vitest --run web/src/lib/break/memory.test.js`. Covers round progression, a wrong press, and the level from sequence length.
  - Preserve: the game is client-only (R7.2).

## Wave 3: browser UI

- [ ] **T13 [S] (25 min) Bot worker and the bot page**
  - Files: `web/src/lib/break/bot.worker.js`, `web/src/lib/break/Board.svelte` (shared board wrapper), `web/src/routes/break/bot/+page.svelte`.
  - The worker wraps `chooseMove`. The page lets the user pick White or Black, plays moves with `chess.js`, shows the text status, and offers to save a result when signed in. Include a random-move fallback with text when the worker fails.
  - Depends on: T10, T06 (for `api.js`).
  - Verify: manual check in a browser with the dev server (Brighton runs it), plus a unit check that the page imports no GSAP.
  - Preserve: R6.1, R6.5 to R6.7, R8.5.

- [ ] **T14 [S] (25 min) Daily puzzle page**
  - Files: `web/src/routes/break/puzzle/+page.svelte`, `web/src/lib/break/api.js`.
  - Fetch the daily puzzle, show the board with `Board.svelte`, accept clicks and typed UCI or SAN, show feedback as text, and offer score saving when signed in. Show a text message for `not_imported`.
  - Depends on: T06, T07, T11, T13 (shared board).
  - Verify: manual browser check of a solved puzzle and a wrong move. Confirm that the save call sends no score number.
  - Preserve: R3.4, R4.4, R4.5.

- [ ] **T15 [S] (25 min) Memory game page with keyboard and reduced motion**
  - Files: `web/src/routes/break/memory/+page.svelte`.
  - Grid of nine buttons with a roving tab index. Arrow keys, Enter and Space. `aria-live` region. CSS-only transitions. A `prefers-reduced-motion` branch shows the sequence as text.
  - Depends on: T12.
  - Verify: keyboard-only play (Brighton checks with the keyboard). Under reduced motion, no transition runs.
  - Preserve: R7.4 to R7.6, R8.2 to R8.4, R8.6. No GSAP.

- [ ] **T16 [S] (20 min) Hub page and the single navigation entry**
  - Files: `web/src/routes/break/+page.svelte`, `web/src/lib/break/NavEntry.svelte`, and one import line in the app shell (the only change outside `/break`).
  - The hub shows three cards with text labels, the puzzle status (or the `not_imported` message), and the last saved scores when signed in.
  - Depends on: T08, T13, T14, T15.
  - Verify: the hub renders with and without an imported puzzle. The only change to the app shell is one import and one `<NavEntry />`.
  - Preserve: R1.3, R8.4.

## Wave 4: checks and cut readiness

- [ ] **T17 [S] (25 min) Isolation and licence check script**
  - Files: `deploy/breakroom/check-isolation.sh` (or a CI step in `.github/workflows/ci.yml`, one job).
  - Fail the build if a file outside the isolation map imports `breakroom`, `$lib/break`, `chess.js` or `cm-chessboard`, except `NavEntry.svelte` and the single router mount. Fail if `chessground`, `stockfish` or any `AGPL`/`GPL` licence appears in `web/package-lock.json` for chess packages.
  - Depends on: T16.
  - Verify: the script passes on the current tree, and fails when a test import is added to a scratch file (then removed).
  - Preserve: R1.5, R2.4.

- [ ] **T18 [S] (25 min) Bundle and motion check**
  - Files: no source change expected. Add a check to the build verification notes in `deploy/breakroom/README.md`.
  - Run the production build. Confirm `gsap` is absent from `/break` chunks, and `chess.js` and `cm-chessboard` are absent from the landing and IDE chunks.
  - Depends on: T16.
  - Verify: the build output is inspected with a search for the package names in each chunk. Record the result.
  - Preserve: R9.2, R8.1.

- [ ] **T19 [S] (25 min) Cut rehearsal on a scratch branch**
  - Files: `deploy/breakroom/README.md` (cut procedure, R11.1).
  - On a scratch branch, follow the cut procedure: delete the three break-room paths, remove the router mount line, remove `NavEntry` and its import, remove the two chess packages and their notices. Then run the build and the Rust checks.
  - Depends on: T17, T18.
  - Verify: the build and `cargo test -p counterspell-server` pass after the cut. Record the exact commands in the README. Discard the scratch branch afterwards.
  - Preserve: R1.4 and R11.2 (the tables stay).

- [ ] **T20 [S] (15 min) Record the break-room decision for the release checklist**
  - Files: `deploy/breakroom/README.md` (status section only).
  - State whether the feature shipped, and list any cut or deferred items, so the freeze-day check can read it.
  - Depends on: T19.
  - Verify: the README status matches the current tree.
  - Preserve: the release checklist rule to record any failed check and the decision to fix, defer or cut.
