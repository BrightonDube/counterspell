# Low-level design: break-room

## Design approach

This is a Quick Spec for the lowest-priority feature. It turns the break-room notes in `docs/system-design.md` (§0, §4, §9 API rows, and the F10 row in §1.1) and `docs/requirements.md` (E10 BRK-1 to BRK-3) into an implementable plan. The approach is to keep the feature small, isolated and deletable:

- Server code lives in one module inside `counterspell-server`. No new Cargo package is added, because `docs/system-design.md` fixes the six packages.
- Puzzle data is imported once with SQL. The server never reads the CSV at request time.
- Both game logics that matter for trust (puzzle answers and bot moves) run in the browser, because they are first-party code and nothing is graded. The server only validates and stores cosmetic scores.
- The UI is one route, `/break`, with no GSAP and no ambient motion.

## Scope and non-goals

### Included

- A `puzzles` table and a `game_scores` table, in one migration.
- A one-time import of about 3,000 puzzles from the CC0 Lichess CSV, using SQL.
- Server endpoints: daily puzzle, save score, list own scores.
- Browser pages: hub, daily puzzle, chess bot, memory game.
- Dependencies: `chess.js` and `cm-chessboard`.

### Deferred or excluded

- Duels and multiplayer, which are "Later" in the requirements.
- Puzzle streaks, ratings or leaderboards.
- Server-side bot or Stockfish.
- Dashboard integration with `game_scores` (the dashboard spec may read it later).
- FSRS, Night Shift and payments.

## Isolation map

Everything the feature touches is listed here. Anything outside this list is a bug in the feature.

```text
crates/server/src/breakroom/          # new module inside counterspell-server (no new crate)
├── mod.rs                            # pub fn router() -> Router, mounted once
├── daily.rs                          # pure: sast_date(), daily_index(); repo: count, fetch by offset
├── scores.rs                         # tagged request enum, pure score_for(), validation, repo insert/list
└── handlers.rs                       # axum handlers: daily, post score, list scores
migrations/<timestamp>_breakroom.sql  # puzzles + game_scores (one file)
deploy/breakroom/
├── import-puzzles.sql               # one-time import from the Lichess CSV
└── README.md                         # import steps and the cut procedure (R11)
web/src/routes/break/
├── +page.svelte                      # hub: three cards (puzzle, bot, memory)
├── puzzle/+page.svelte               # daily puzzle page
├── bot/+page.svelte                  # chess bot page
└── memory/+page.svelte               # memory game page
web/src/lib/break/
├── api.js                            # fetch helpers for /api/breakroom (uses the auth CSRF header)
├── puzzle.js                         # pure: play-through state machine over chess.js
├── minimax.js                        # pure: chooseMove(fen, options), eval, move ordering
├── bot.worker.js                     # Web Worker wrapper around minimax.js
├── memory.js                         # pure: round generation, input checking, level
├── Board.svelte                      # cm-chessboard wrapper, text alternative, move input
└── NavEntry.svelte                   # the one navigation component that links to /break
web/package.json                      # + chess.js, + cm-chessboard (exact pinned versions)
THIRD-PARTY-NOTICES.md                # + chess.js (BSD-2-Clause), + cm-chessboard (MIT)
crates/server/tests/fixtures/breakroom-sample.csv   # synthetic 12-row fixture, labelled as synthetic
```

Two single-line hooks connect the feature to the rest of the system:

1. Server router: one `.nest("/api/breakroom", breakroom::router(state))` line in the router assembled by the server (see the route order in `repo-skeleton`).
2. Web: one `<NavEntry />` in the app shell. `NavEntry.svelte` holds the link to `/break`.

## Data model

Both tables follow `docs/system-design.md` §4. The only additions are a surrogate `id` on `game_scores` and provenance columns on `puzzles`.

```sql
CREATE TABLE puzzles (
  id          text PRIMARY KEY,                 -- Lichess PuzzleId, for example "00008"
  fen         text NOT NULL CHECK (fen <> ''),
  moves       text NOT NULL CHECK (char_length(moves) BETWEEN 1 AND 500),  -- UCI, space separated; moves[0] is the opponent's move
  rating      integer NOT NULL CHECK (rating BETWEEN 400 AND 3500),
  themes      text[] NOT NULL DEFAULT '{}',
  popularity  integer NOT NULL,
  nb_plays    integer NOT NULL,
  source_date date NOT NULL,                    -- build date of the CSV, for provenance
  imported_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX puzzles_rating ON puzzles (rating);

CREATE TABLE game_scores (
  id         bigserial PRIMARY KEY,
  user_id    uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
  game       text NOT NULL CHECK (game IN ('chess_puzzle', 'chess_bot', 'brain')),
  score      integer NOT NULL CHECK (score BETWEEN 0 AND 10000),
  meta       jsonb NOT NULL DEFAULT '{}'::jsonb CHECK (octet_length(meta::text) <= 1024),
  created_at timestamptz NOT NULL DEFAULT now()
);
CREATE INDEX game_scores_user_game ON game_scores (user_id, game, created_at DESC);
-- At most one puzzle score per user per SAST date (R5.5).
CREATE UNIQUE INDEX game_scores_one_puzzle_per_day
  ON game_scores (user_id, (meta->>'date'))
  WHERE game = 'chess_puzzle';
```

**Why `ON DELETE CASCADE` on `game_scores`:** the auth spec already expects a `game_scores` table and deletes per-user rows through foreign keys to `users(id)` (auth design, demo reset). The cascade means account deletion (ACC-5) removes these rows with no extra code. A test covers it (R10.4).

**Why `meta` is checked with `octet_length(meta::text)`:** a `CHECK` constraint must use immutable expressions. The jsonb-to-text cast is immutable, so this check is safe. The server also enforces the same limit first, so the database rarely sees an oversized value.

## Import (one time, SQL)

`deploy/breakroom/import-puzzles.sql` is run once against the production database after the migration. The CSV is the Lichess `lichess_db_puzzle.csv` file, decompressed locally first (`zstd -d`). The import never touches the network.

Steps the script performs:

1. Create a temporary staging table with the CSV's columns: `PuzzleId, FEN, Moves, Rating, RatingDeviation, Popularity, NbPlays, Themes, GameUrl, OpeningTags`.
2. `\copy` the CSV into the staging table, with `CSV HEADER`.
3. `INSERT INTO puzzles ... SELECT ... FROM staging WHERE Rating BETWEEN 1000 AND 2200 AND Popularity >= 80 ORDER BY Popularity DESC, PuzzleId LIMIT 3000 ON CONFLICT (id) DO NOTHING`.
4. Set `source_date` from a psql variable the operator supplies (the CSV build date).
5. Drop the staging table.

Selection values (rating band, popularity floor and the 3,000 cap) are assumptions for launch (Open question 3). Re-running the script is safe because of `ON CONFLICT DO NOTHING`. Adding puzzles later changes the sorted order, so the daily pick can change. That is acceptable, and the README says so.

The fixture `crates/server/tests/fixtures/breakroom-sample.csv` has the same header with 12 hand-written, clearly labelled synthetic rows. It is used only by the import test, never shipped as puzzle content.

## Daily puzzle

### Calendar

Africa/Johannesburg is UTC+2 all year, so no timezone database is needed:

```text
sast_date(utc) = (utc + 2 hours).date()
```

### Selection

```text
days  = (sast_date - 1970-01-01).whole_days()      // i64, may be negative before 1970 but not in practice
count = SELECT count(*) FROM puzzles               // 0 means not_imported
index = days.rem_euclid(count)                     // 0 <= index < count
row   = SELECT id, fen, moves, rating, themes FROM puzzles ORDER BY id OFFSET index LIMIT 1
```

`rem_euclid` keeps the index non-negative. The pure function `daily_index(days, count) -> Option<i64>` returns `None` when `count == 0`, so the handler can map that to `not_imported` without a panic.

### Response

```json
{
  "date": "2026-11-13",
  "puzzle": {
    "id": "00008",
    "fen": "…",
    "moves": "e2e4 e7e5 …",
    "rating": 1420,
    "themes": ["mate", "mateIn2"]
  }
}
```

The client receives the solution moves. Puzzles are public on Lichess, and nothing is graded. This is Open question 1.

**Not cached.** The count and the offset query on about 3,000 rows are cheap. Valkey is not needed here. If p95 misses 100 ms (R9.1), add a Valkey cache keyed by the SAST date.

## Scores

### Request shape

One tagged enum, with `deny_unknown_fields` on every variant:

```json
{ "game": "chess_puzzle", "date": "2026-11-13", "solved": true, "mistakes": 2 }
{ "game": "chess_bot", "result": "win", "moves": 41 }
{ "game": "brain", "level": 6 }
```

### Scoring rules (pure function `score_for`)

| Game | Valid input | Stored score | Meta stored |
|---|---|---|---|
| `chess_puzzle` | `solved` must be `true`; `mistakes` 0 to 20; `date` must equal today's SAST date | `max(10, 100 - 15 × mistakes)` | `{"date": "YYYY-MM-DD", "mistakes": n}` |
| `chess_bot` | `result` in `win`, `draw`, `loss`; `moves` 1 to 400 | win 100, draw 50, loss 0 | `{"result": "...", "moves": n}` |
| `brain` | `level` 1 to 99 | `min(990, level × 10)` | `{"level": n}` |

A `chess_puzzle` request with `solved: false` returns `422 not_solved`. Nothing is stored, because an unsolved puzzle is not a score.

The bot result and the brain level are client-reported. They are cosmetic, so the design accepts that a player can fake them. Rate limiting (30 per hour per user) keeps the table small. The tables never feed rankings or judging.

### Responses

| Case | Status | Body |
|---|---|---|
| Stored | 201 | `{ "id": 123, "score": 100 }` |
| Puzzle already stored for the date | 200 | `{ "status": "already_recorded" }` |
| Not signed in | 401 | `{ "code": "unauthenticated" }` (from auth) |
| Bad shape or unknown field | 400 | `{ "code": "invalid_body", "message": "..." }` |
| Wrong puzzle date | 422 | `{ "code": "wrong_date" }` |
| Puzzle not solved | 422 | `{ "code": "not_solved" }` |
| Meta too large | 422 | `{ "code": "meta_too_large" }` |
| Rate limited | 429 | `{ "code": "rate_limited", "retry_after_seconds": n }` |

The "already recorded" insert uses `INSERT ... ON CONFLICT DO NOTHING` against the partial unique index. It then reads back the id if the row is new. The index is what enforces R5.5, not the handler.

### Listing

`GET /api/breakroom/scores` returns the signed-in user's latest 10 rows, ordered by `created_at DESC`. The query always filters by the `CurrentUser` id from the session extractor, so no user can request another user's rows.

## Routes

| Method and path | Auth | Purpose |
|---|---|---|
| `GET /api/breakroom/puzzle/daily` | none | today's puzzle (R3) |
| `POST /api/breakroom/scores` | session + CSRF | save an outcome (R5) |
| `GET /api/breakroom/scores` | session | own latest 10 scores (R5.9) |

These three routes are the whole server surface. The system design's `/api/breakroom/puzzle/daily` and `/api/breakroom/scores` rows are kept as written.

## Chess bot

### Algorithm

`minimax.js` exports a pure function:

```text
chooseMove(fen, { depth: 2, nodeBudget: 60000, deadlineMs: 1000, rng }) -> { uci, san, nodes, timedOut }
```

- Uses `chess.js` to generate legal moves (`moves({ verbose: true })`) and to make and undo moves.
- Evaluation: material only, with the usual piece values (pawn 100, knight 320, bishop 330, rook 500, queen 900), from the side to move's point of view. Checkmate scores ±100000 minus the ply depth, so the bot prefers faster mates.
- Search: negamax with alpha-beta pruning, depth 2 plies (the bot's move and the reply).
- Move ordering: captures first, ordered by victim value, then the rest in the order `chess.js` returns them. This keeps the search fast and deterministic.
- Ties: broken by the injected `rng`, which is seeded by the caller. The page seeds it with the game id, so the bot is repeatable in tests.
- Stops on either the node budget (deterministic) or `deadlineMs` (safety for slow devices). If the search stops early, the result uses the best move found so far, and `timedOut: true` is returned for the UI.

Depth 2 with about 30 moves each gives roughly a few thousand node evaluations per move. The node budget keeps the decision deterministic. Wall-clock time only guards against slow devices.

### Worker

`bot.worker.js` receives `{ fen, seed }`, calls `chooseMove`, and posts back the result. The page keeps the game state with `chess.js` and sends a new FEN to the worker each turn. The worker never touches the DOM.

## Puzzle play

`puzzle.js` is a pure state machine:

```text
state = { fen, solution: string[], index, mistakes, status: 'playing' | 'solved' | 'failed' }
tryMove(state, uci) -> { state', feedback }
```

- The first solution move is the opponent's move. It is played automatically when the puzzle starts (R3.6).
- Each user move is compared with the next solution move, in UCI form, after `chess.js` confirms the move is legal (R4.1).
- Correct: play the next opponent reply (if there is one) and advance. Wrong: increment `mistakes` and return the message "Not the solution. Try again." (R4.3).
- The puzzle is solved when the index reaches the end of the solution list (R4.5).

Move input: the board click path and a text input accept UCI (`e2e4`) and SAN (`Nf3`). The text input is the keyboard path, because cm-chessboard's move input is pointer based (R8.6).

Illegal moves (rejected by `chess.js`) show a message and do not count as mistakes. The stored `mistakes` value is capped at 20 by both the client and the server.

## Memory game

`memory.js` is a pure module:

```text
newRound(sequence, rng) -> sequence + one random tile index in 0..8
checkPress(sequence, presses, pressIndex) -> 'continue' | 'round_done' | 'wrong'
level(sequence) -> sequence.length - 2       // starts at 3 tiles, so level 1 is the first finished round
```

The page drives it:

1. Light the sequence one tile at a time, with a 600 ms gap. Under reduced motion, show the sequence as a text list instead, and skip the timing (R8.2).
2. Wait for presses. Each press calls `checkPress`.
3. On `round_done`, add a tile and repeat. On `wrong`, end the game and show the level (R7.6).

Keyboard: arrow keys move focus across the 3 × 3 grid (a roving tab index). Enter and Space press the focused tile. An `aria-live` region announces each lit tile, each correct press and the final level (R7.4 and R7.5).

Colours: lit tile uses the candle-amber token. A correct press flashes the pass-green token briefly. A wrong press uses the failure-red token. The game also shows text labels ("Correct", "Wrong, game over at level 4").

## Frontend

- `/break` hub is prerendered (adapter-static). The puzzle, bot and memory pages are client-only, because they depend on the browser and the Web Worker.
- `NavEntry.svelte` is the only file outside `/break` that imports from `$lib/break`. It only renders a link.
- `cm-chessboard` is styled with CSS variables mapped to the Steel Night tokens. Squares use the steel and moss tones from `docs/palette.md`. The last move is highlighted in candle amber. No purple, and no green or red for board states.
- All motion is CSS transitions on tile background and piece movement. The `@media (prefers-reduced-motion: reduce)` block removes both. GSAP is not imported (R8.1).
- The board wrapper has a visible `<output>` text line for FEN, last move and turn (R8.5).

## Security

| Threat | Mitigation |
|---|---|
| Forged scores | Accepted as cosmetic. Scores are rate limited (R5.8), cannot be back-dated (R5.6), are never ranked, and are never used for judging. |
| Puzzle date spoofing | The server checks the date against its own SAST clock. |
| Oversized or hostile JSON | `deny_unknown_fields`, bounded `meta`, typed fields, and the 64 KB request body limit from the server. |
| Cross-user reads | `GET` filters by the session user id only. |
| Stored XSS via external text | Themes and puzzle ids are rendered as text. |
| CSRF on writes | The auth CSRF middleware wraps `/api`, so the `POST` is covered (R10.6). |
| Supply chain | Only two chess packages, both pinned, both in the notices. A CI check fails if `chessground` or `stockfish` appear in `package-lock.json` (see tasks). |
| Account deletion leaves rows | `ON DELETE CASCADE` plus a test. |

## Error handling

- Empty puzzles table: `503 not_imported`. The hub shows "Today's puzzle is not available yet" and still shows the bot and memory game.
- Worker failure: the bot page falls back to a random legal move and shows "Bot is resting, a simple move was played", in text.
- Network failure on save: the page keeps the result on screen and offers a retry button. No retry loop runs in the background.

## Testing approach

Verification only. These checks support the implementation tasks and add no product behaviour.

- **Rust unit tests** (`daily.rs`, `scores.rs`): `sast_date` around midnight UTC (23:59 and 00:01), `daily_index` never out of range for any `count >= 1` (a property test over random day numbers and counts), `daily_index` returns `None` for `count == 0`, and `score_for` edge values (mistakes 0, 20, 21 rejected; level 0, 1, 99, 100 rejected).
- **Rust integration tests** (`crates/server/tests/breakroom_*.rs`, against a throwaway Postgres): import of the synthetic fixture, the daily endpoint, the duplicate puzzle score, the foreign-key cascade on account deletion.
- **JavaScript unit tests** (`vitest --run`): `minimax.js` takes a free mate in one, returns a legal move in every position from a fixed list, and is repeatable with the same seed. `puzzle.js` accepts the solution, rejects a wrong move, and counts mistakes. `memory.js` handles round progression and a wrong press.
- **Bundle check**: the built `build/` output contains no `gsap` import in `/break` chunks, and `chess.js` and `cm-chessboard` do not appear in the landing or IDE chunks.
- **Isolation check** (a script in CI): no file outside the isolation map imports `breakroom`, `$lib/break`, `chess.js` or `cm-chessboard`, except `NavEntry.svelte` and the router mount.
- **Cut rehearsal**: on a scratch branch, follow the cut procedure, then run the build and tests. The build must pass.

## Rust lesson (for implementers)

After each Rust change, the implementer adds a 3–5 line Rust lesson to the hand-off note, as `rust-teaching.md` requires. Example topics for this module: `Option` from `daily_index` replacing an `unwrap` on an empty table, the `?` operator with a typed error that maps to a JSON error code, and why `rem_euclid` avoids negative indexes.

## Alternatives considered

| Decision | Chosen | Alternative | Why |
|---|---|---|---|
| Bot runs | Browser Web Worker | Server endpoint | No server load, no external calls, and the bot is first-party code. |
| Puzzle check | Browser (solution sent) | Server check per move | Simpler, and the solution is public on Lichess. Trade-off: a player can read the solution. Open question 1. |
| Board library | cm-chessboard (MIT) | chessground (GPL) | Licence rule forbids GPL. |
| Engine | Own minimax, depth 2 | Stockfish.js | Stockfish is GPL and too heavy for a break. |
| Import | SQL script | Rust subcommand | No new code path in the server, and the import is reviewable as SQL. |
| Daily pick | Day number mod count | Seeded hash of the date | Simpler to explain and test, and it needs no seed. |
| Bot search | Node budget plus deadline | Wall clock only | Node budget gives the same move each time. |
| Cut mechanism | Delete paths and two hooks | Compile-time Cargo feature | A feature flag adds a permanent build axis for a feature that is probably never cut in part. Open question 7. |

## Risks

| Risk | Effect | Mitigation |
|---|---|---|
| Lichess changes the CSV columns | Import fails | The import script names every column, and the README records the file date. |
| Rating band gives puzzles that are too hard for the audience | Users quit | Open question 3. Adjust the band in the import script before import. |
| Bot is too weak or too slow on a phone | Poor experience | Node budget, deadline, text fallback, depth fixed at 2. |
| Break room grows into judging scope | Time loss before freeze | Stop at the isolation map. Anything outside it is out of scope. |
| Scores get faked and look wrong | Cosmetic only | Documented as unranked. |

## Open questions

Each question includes the choice this spec assumed. Brighton can change any of them before implementation.

1. **Where is the puzzle solution checked?** Assumed: in the browser. The daily endpoint returns the solution moves, and the server only stores the outcome. The alternative is a server check per move, which is stricter but costs more code and round trips.
2. **Does playing the puzzle need sign-in?** Assumed: no. Playing and reading the daily puzzle are public. Saving a score needs sign-in, because the score is user data.
3. **Puzzle selection.** Assumed: rating 1000 to 2200, popularity at least 80, sorted by popularity, capped at 3,000 puzzles. Change the numbers in `import-puzzles.sql` before the import runs.
4. **Bot strength and colour.** Assumed: depth 2, material only, and the user chooses White or Black. A stronger bot needs a deeper search and more time, so the depth is fixed for the launch.
5. **Which brain game.** Assumed: a 3 × 3 "Lantern recall" memory game, because it is small and needs no assets. The alternative is a pattern-matching game.
6. **Navigation.** Assumed: one link in the app shell, through `NavEntry.svelte`. Nothing on the landing page links to `/break`, so the landing page stays calm.
7. **Cut mechanism.** Assumed: delete the paths and the two hooks, with no runtime or compile-time flag. The alternative is a `breakroom` Cargo feature, which makes the cut a single switch but adds a permanent build option.
8. **Daily boundary.** Assumed: the daily puzzle changes at 00:00 SAST (UTC+2). A player who solves just before midnight and saves after midnight gets `wrong_date`. The alternative is to accept yesterday's date for ten minutes, which adds a rule for little gain.
9. **Bot results when signed out.** Assumed: they are not saved and the page says so.
10. **Demo account.** Assumed: the judge demo account can use the whole feature. It has no extra restrictions, and the demo reset clears its `game_scores` rows through the existing cascade rule.
11. **Hints in puzzles.** Assumed: one text hint per puzzle, built from the Lichess themes (for example "Look for a mate"). The alternative is no hints, which is simpler.
12. **Migration timestamp.** Assumed: the break-room migration runs after the auth migration, because `game_scores` references `users`. The exact timestamp is set by `sqlx migrate add` when the task runs.
