# Requirements: break-room

## Purpose

Give learners a short, optional break inside Counterspell: one daily chess puzzle, a simple chess bot, and one small memory game. The feature is a stretch goal (system design §14, requirements §8 "Could"). It starts only after the core is freeze-safe, and it is the first feature to cut. It must not change how judging, the IDE, auth, or the dashboard behave.

## Scope constraints

- All break-room code lives in the isolated locations listed in `design.md` (`web/src/routes/break/`, `web/src/lib/break/`, `crates/server/src/breakroom/`, `deploy/breakroom/`, and one break-room migration). Removing those paths and the two single-line hooks listed in `design.md` removes the feature.
- The feature SHALL NOT add a Cargo package. It lives inside `counterspell-server`.
- Chess puzzle data SHALL come only from the CC0 Lichess puzzle CSV. No Lichess AGPL code or `lichess-org/database` code SHALL be copied or linked.
- The chess bot SHALL NOT use Stockfish or any GPL/AGPL chess engine. The board SHALL NOT use chessground (GPL).
- No break-room feature SHALL execute untrusted code. The bot and the memory game run in the browser as first-party code. Nothing in this feature runs in the judge sandbox.
- No break-room feature SHALL call Lichess, OpenAI, Groq, or any other external service at request time.
- Break-room features SHALL NOT be a second judging system. Scores are cosmetic and unranked.

## EARS acceptance criteria

### Isolation and cut-ability (R1)

1. THE SYSTEM SHALL keep every break-room server route under the single `/api/breakroom` prefix, mounted by one `nest` line in the server router.
2. THE SYSTEM SHALL keep every break-room page under the single `/break` route, and every break-room component under `web/src/lib/break/`.
3. THE SYSTEM SHALL keep the break-room navigation entry in one component, so the entry can be removed without editing other components.
4. IF the `/break` directory, the `/api/breakroom` mount, and the navigation entry are removed, THEN THE SYSTEM SHALL still build and pass its existing checks with no other changes.
5. THE SYSTEM SHALL NOT import break-room modules from any module outside the break-room paths, except the single router mount and the single navigation entry.

### Licence and data source (R2)

1. THE SYSTEM SHALL import puzzles only from the Lichess puzzle CSV, which is CC0 (verified on the Lichess database page and the public Hugging Face mirror, 12 Nov 2026).
2. THE SYSTEM SHALL import puzzles with SQL in `deploy/breakroom/import-puzzles.sql`, not with code copied from the `lichess-org/database` repository.
3. THE SYSTEM SHALL add `chess.js` (BSD-2-Clause) and `cm-chessboard` (MIT) as the only chess-related npm dependencies, and SHALL record both in `THIRD-PARTY-NOTICES.md`.
4. IF a proposed chess dependency is GPL or AGPL, including Stockfish and chessground, THEN THE SYSTEM SHALL refuse to add it and report the conflict.
5. THE SYSTEM SHALL record the CSV build date in `puzzles.source_date` for provenance.

### Daily chess puzzle (R3)

1. WHEN a client requests `GET /api/breakroom/puzzle/daily`, THE SYSTEM SHALL return the same puzzle to every user for the same Africa/Johannesburg calendar date (UTC+2, no daylight saving time).
2. THE SYSTEM SHALL choose the daily puzzle with `days_since_1970_01_01(sast_date) mod count`, indexed over puzzles ordered by `id`, so the choice is deterministic and needs no stored state.
3. THE SYSTEM SHALL return the FEN, the moves in UCI form, the rating, the themes, the puzzle id and the SAST date, and SHALL NOT return anything the client does not need to play the puzzle.
4. THE SYSTEM SHALL NOT require sign-in to read the daily puzzle.
5. IF the `puzzles` table is empty, THEN THE SYSTEM SHALL return HTTP 503 with code `not_imported` and SHALL NOT return an error page.
6. WHEN the client plays the puzzle, THE SYSTEM SHALL play the first UCI move automatically as the opponent's move, then ask the user for the reply moves, as the Lichess format specifies.

### Puzzle play (R4)

1. THE SYSTEM SHALL check each user move for legality with `chess.js` in the browser.
2. WHEN a user move matches the next solution move, THE SYSTEM SHALL play the opponent's reply and continue the puzzle.
3. IF a user move does not match, THEN THE SYSTEM SHALL show a text message that says the move is not the solution, count one mistake, and let the user try again.
4. THE SYSTEM SHALL accept moves as board clicks or as typed UCI or SAN text, so the puzzle can be played from the keyboard.
5. WHEN the puzzle is solved, THE SYSTEM SHALL show a text result and offer to save the score when the user is signed in.

### Score recording (R5)

1. WHEN a signed-in user saves a break-room score, THE SYSTEM SHALL check the user's session with the auth `AuthUser` extractor and SHALL write the row for that user only.
2. THE SYSTEM SHALL compute the stored score on the server from the submitted outcome. The client SHALL NOT send a score number.
3. THE SYSTEM SHALL accept `game` values only from `chess_puzzle`, `chess_bot` and `brain`.
4. THE SYSTEM SHALL reject unknown fields with code `invalid_body`.
5. THE SYSTEM SHALL record at most one `chess_puzzle` score per user per SAST date. A repeat returns HTTP 200 with `status: "already_recorded"`.
6. IF a puzzle score is for a date other than today's SAST date, THEN THE SYSTEM SHALL reject it with code `wrong_date`, so scores cannot be back-dated.
7. THE SYSTEM SHALL cap a break-room `meta` value at 1 KB, and SHALL reject larger values with code `meta_too_large`.
8. WHERE a user saves more than 30 break-room scores in an hour, THE SYSTEM SHALL return HTTP 429 with the wait time, using the Valkey rate limiter from auth.
9. THE SYSTEM SHALL return only the signed-in user's own scores from `GET /api/breakroom/scores`, limited to the latest 10.

### Chess bot (R6)

1. THE SYSTEM SHALL run the bot in a Web Worker in the browser, so the UI stays responsive.
2. THE SYSTEM SHALL choose bot moves with minimax and alpha-beta pruning, searching 2 plies, with material as the only evaluation term.
3. THE SYSTEM SHALL limit each bot decision to a fixed node budget, so that the same position gives the same move, and SHALL also stop the search if it exceeds 1000 ms of wall-clock time.
4. THE SYSTEM SHALL play only legal moves, as generated by `chess.js`.
5. WHEN the game ends by checkmate, stalemate, threefold repetition, the fifty-move rule or insufficient material, THE SYSTEM SHALL show the result in text.
6. WHERE the user is signed in and the game ends, THE SYSTEM SHALL offer to save the result as `chess_bot` with `result` set to `win`, `draw` or `loss`.
7. THE SYSTEM SHALL NOT call any server for bot moves.

### Memory game (R7)

1. THE SYSTEM SHALL present a 3 × 3 grid of tiles. Each round, the game lights one more tile than the previous round, and the user must repeat the sequence in order.
2. THE SYSTEM SHALL run the game entirely in the browser, with no network call during play.
3. THE SYSTEM SHALL score the level reached, where the level is the number of fully repeated rounds, and SHALL save it as `brain` only when the user is signed in and chooses to save.
4. THE SYSTEM SHALL support keyboard play: arrow keys move focus across tiles, and Enter or Space presses the focused tile.
5. THE SYSTEM SHALL announce each lit tile, each correct press and the game-over result through an `aria-live="polite"` region.
6. IF the user presses a wrong tile, THEN THE SYSTEM SHALL end the game and show the level reached.

### Motion, accessibility and visual rules (R8)

1. THE SYSTEM SHALL NOT import GSAP anywhere under `/break`. Break-room motion uses short CSS transitions only, as `web/` rules limit GSAP to the landing page and the Familiar.
2. WHERE `prefers-reduced-motion: reduce` is active, THE SYSTEM SHALL disable tile transitions and show the memory sequence as a static text list.
3. THE SYSTEM SHALL use only semantic colour tokens from `web/src/app.css`. Bright green SHALL appear only for a correct pass, and red SHALL appear only for a failure.
4. THE SYSTEM SHALL NOT rely on colour alone: every board state and game result SHALL also have a text label.
5. THE SYSTEM SHALL give the chess board a text alternative: the current FEN, the last move in text, and the whose-turn status.
6. THE SYSTEM SHALL be fully usable by keyboard, with visible focus on every control.

### Performance and bundle (R9)

1. WHEN the daily puzzle is requested, THE SYSTEM SHALL answer within 100 ms p95 on the VPS, using the indexed `puzzles` table [E].
2. THE SYSTEM SHALL load the `/break` code and `chess.js` and `cm-chessboard` only on `/break` routes, and SHALL NOT add them to the landing page or IDE bundles.
3. THE SYSTEM SHALL store at most about 3,000 puzzles in the `puzzles` table at launch [E].

### Security and data (R10)

1. THE SYSTEM SHALL validate every break-room request body on the server before it writes to the database.
2. THE SYSTEM SHALL return stable error codes in JSON and SHALL NOT include stack traces or SQL text.
3. THE SYSTEM SHALL render puzzle themes, puzzle ids and any other external text as plain text.
4. THE SYSTEM SHALL delete a user's `game_scores` rows when the user account is deleted, through the `ON DELETE CASCADE` foreign key, and SHALL add a test that proves it.
5. THE SYSTEM SHALL store no personal data in `puzzles`, and SHALL store only `user_id` in `game_scores`.
6. THE SYSTEM SHALL apply the CSRF protection from the auth spec to every break-room `POST`.

### Cut procedure (R11)

1. THE SYSTEM SHALL document, in `deploy/breakroom/README.md`, the exact steps to remove the feature: delete the three break-room paths, remove the router mount, remove the navigation entry, and remove the two chess npm packages and their notices.
2. WHERE the feature is cut, THE SYSTEM SHALL keep the `puzzles` and `game_scores` tables. Dropping them is a separate, optional migration that Brighton approves.
