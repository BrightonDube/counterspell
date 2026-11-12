# Tasks: hints

All tasks are implementation tasks for `design.md` and `requirements.md`. No task in this document is run by the spec-generation step. Each task is sized for 30 minutes or less. `[P]` means the task can run in parallel with others in the same wave once its dependencies are done. `[S]` means it is sequential.

Conventions for every Rust task (same as `run-lifecycle`):

- Follow `.kiro/steering/rust-teaching.md`, `security.md`, `api-conventions.md` and `structure.md`. No `unwrap()` or `expect()` outside tests. No `unsafe`.
- Run `cargo clippy -p <package> --all-targets -- -D warnings` before finishing.
- Every public item has a plain-English doc comment.
- Use runtime-checked `sqlx::query` / `query_as`, not the compile-time macros.
- Database tests are named `pg_…` and marked `#[ignore = "needs DATABASE_URL"]`, with `#[sqlx::test(migrations = "../../migrations")]`. Valkey tests are named `valkey_…` and marked `#[ignore = "needs VALKEY_URL"]`.
- Never read, create or write `.env` files, key files or credentials. Tests use `HintConfig::from_map` with fake values.
- End each Rust task with a **Rust lesson** of 3 to 5 lines in plain English, appended to `docs/kiro-log/rust-lessons.md` as `# Rust lesson: hints Hxx`.
- Any new third-party crate must be added to `THIRD-PARTY-NOTICES.md` in the same task. If `cargo deny check licenses` fails, stop and ask Brighton before editing `deny.toml`.

## Wave 0: shared types and the authored ladder

- [ ] **G1 [P] (25 min) Parse the authored ladder in `counterspell-grimoire`**
  - Files: `crates/grimoire/src/hints.rs`, `crates/grimoire/src/lib.rs` (re-export only).
  - Add `AuthoredLadder` with `parse(toml_or_yaml_section)` returning exactly four rungs in order 1 to 4. Reject a missing, duplicate or extra rung.
  - Requirements: R1.1.
  - Depends on: `grimoire-packs` parser skeleton existing. If it does not, create only this module.
  - Verify: `cargo test -p counterspell-grimoire hints::` with unit tests for a good ladder, a missing rung, a duplicate rung and an extra rung.
  - Preserve: the crate remains the only pack parser; no second parser in the server.
  - Rust lesson: `enum` and `match` for "exactly these cases"; why `Result` beats a panic for bad author input.

- [ ] **G2 [S] (25 min) Validate authored hint text**
  - Files: `crates/grimoire/src/hints.rs`.
  - Enforce: each rung is 1 to 1,200 characters; no fenced or indented code block longer than 3 lines in any rung; no empty rung. Return a list of author-readable errors that name the problem slug and the rung.
  - Requirements: R1.1, R4.2 (authored text obeys the same no-full-solution rule).
  - Depends on: G1.
  - Verify: tests for a 4-line code block (rejected), a 3-line block (accepted), a 1,201-character rung (rejected).
  - Preserve: validation runs before import, so bad text never reaches `problem_hints`.
  - Rust lesson: collecting many errors with `Vec` instead of stopping at the first.

- [ ] **H01 [S] (30 min) Shared hint types in `counterspell-common`**
  - Files: `crates/common/src/hints.rs`, `crates/common/src/lib.rs` (add `pub mod hints;`).
  - Add `HintRung` (newtype over `u8`, `TryFrom` accepts only 1 to 4), `HintSource` (`ladder`, `ai`, `familiar`), `HintRequest` (`submission_id: Uuid`, `rung: HintRung`, `ai: bool`, `#[serde(deny_unknown_fields)]`), `HintView`, `QuotaView`, and the error code list from the design's HTTP table.
  - Requirements: R11.1, R11.2.
  - Depends on: none (uses existing `common` dependencies from `run-lifecycle` T01; if absent, add `serde`, `serde_json`, `uuid`).
  - Verify: `cargo test -p counterspell-common hints::` covers rung bounds and unknown-field rejection.
  - Preserve: `common` has no Axum, sqlx, reqwest or Docker dependency.
  - Rust lesson: newtypes make invalid values unrepresentable; `TryFrom` as the single checked entry point.

## Wave 1: pure logic (no I/O, no database, no network)

- [ ] **H02 [P] (25 min) Configuration parsing**
  - Files: `crates/server/src/hints/config.rs`, `crates/server/src/hints/mod.rs` (declare module).
  - `HintConfig::from_map` parses `HINT_PROVIDERS`, keys, base URLs, timeout, output tokens, limits, spend cap, prices and `AI_STRICT_CONFIG`. `Redacted<String>` prints `[redacted]`.
  - Requirements: R5.1, R5.5, R6.1, R6.2, R8.1, R8.2, R8.4, R11.4.
  - Depends on: H01.
  - Verify: unit tests: default list; empty list disables AI; a listed provider without a key fails when strict and is disabled with a warning when not; `format!("{:?}", key)` contains no secret (P9 part 1).
  - Preserve: no environment access inside the function.
  - Rust lesson: `Option` versus empty string for "not set"; keeping parsing pure so tests need no environment.

- [ ] **H03 [P] (20 min) Ladder gate**
  - Files: `crates/server/src/hints/ladder.rs`.
  - Implement `gate(&BTreeSet<u8>, u8) -> Gate` exactly as in the design.
  - Requirements: R1.2, R1.3.
  - Depends on: H01.
  - Verify: property P1 with `proptest`: for random reveal sets built by valid reveals, the set is a prefix; `gate` never returns `Open` for a rung whose predecessor is missing.
  - Preserve: pure function, no database.
  - Rust lesson: `match` on an enum result; `BTreeSet` gives a stable, ordered set for tests.

- [ ] **H04 [P] (35 min) Prompt builder and output guard**
  - Files: `crates/server/src/hints/guard.rs`.
  - `build_prompt(kind, ctx) -> Prompt` with delimited data sections and size caps (code 8 KB, ghost fields 1 KB with `[cut]`, statement 1 KB). `sanitise_output(text) -> Sanitised { text, removed_code }` removes fenced and indented blocks longer than 3 lines, appends the notice when it removed anything, and cuts to 1,200 characters on a char boundary.
  - Requirements: R3.1–R3.6, R4.1–R4.5.
  - Depends on: H01.
  - Verify: P3 (sanitised output never has a block over 3 lines), P10 (prompt bound under 16 KB for any inputs), P11 (HTML-like text returned verbatim), P12 (canary placed in hidden expected output never appears in prompt; the builder has no parameter for hidden data, so this is a compile-time guarantee plus a test that asserts the canary is absent).
  - Preserve: the builder accepts only ghost row fields and sample data; there is no argument for hidden tests.
  - Rust lesson: slices and `char_indices` for safe cutting; why a function signature can enforce "no hidden data" by leaving the parameter out.

- [ ] **H05 [P] (25 min) Cache key and AST hash**
  - Files: `crates/server/src/hints/cache.rs`.
  - `ast_hash(lang, code)` uses the tree-sitter parser already used for VAL-2, hashes the node kinds in order with SHA-256 (`sha2`), and falls back to hashing the bytes when parsing fails. `cache_key(problem, rung, ast_hash, ghost_input_json)` uses length-separated fields.
  - Requirements: cache identity (design §Cache key), R2.4.
  - Depends on: H01. Uses the `tree-sitter` and `sha2` crates from `run-lifecycle`; add to notices if new.
  - Verify: P6 (deterministic; changes with any input; whitespace-only and comment-only edits give the same AST hash for Python and JavaScript).
  - Preserve: no execution of user code; parsing only.
  - Rust lesson: why length prefixes stop two different inputs from making the same bytes; `Option` for the fallback path.

- [ ] **H06 [P] (25 min) Fallback rule**
  - Files: `crates/server/src/hints/chain.rs`.
  - Implement `next_step` exactly as in the design, plus a `run_chain` helper generic over a `FnMut(usize) -> Future<Result<T, ProviderError>>` that calls providers in order and stops at the first success.
  - Requirements: R5.2, R5.3, R5.4.
  - Depends on: H01.
  - Verify: P4 with `proptest` over outcome lists of length 0 to 3: call count is 0 for no providers, 1 when the first succeeds, and never more than the list length; `Exhausted` only when every call failed.
  - Preserve: no timers in this module; the timeout is applied by the caller.
  - Rust lesson: generics with a closure for "call the next one"; returning a plain enum that the compiler checks.

- [ ] **H07 [P] (25 min) Cost arithmetic**
  - Files: `crates/server/src/hints/cost.rs`.
  - `cost_micros`, `worst_case_micros`, `estimate_tokens` using integer micro-dollars. A helper converts micro-dollars to a SQL-ready decimal string for `cost_usd numeric(10,6)`.
  - Requirements: R6.3, R7.2, R7.3.
  - Depends on: H02 (the `Price` type comes from config).
  - Verify: P8 with `proptest`: worst case ≥ actual for any token counts inside the prompt bound; rounding never returns a negative cost; the decimal string has at most 6 places.
  - Preserve: no float arithmetic for money.
  - Rust lesson: integer maths for money; why `ceil` and `checked_mul` protect the totals.

## Wave 2: I/O adapters

- [ ] **H08 [S] (35 min) Provider client and scripted transport**
  - Files: `crates/server/src/hints/provider.rs`, `crates/server/src/testkit.rs` (feature `testkit`, `ScriptedTransport`).
  - `ChatTransport` trait, `ReqwestTransport` (one shared client, per-call timeout), `OpenAiCompatible`, `ProviderError`. Request and response JSON as in the design. Provider error bodies are dropped; only the status code is kept.
  - Requirements: R5.1, R5.2, R7.2, R8.3.
  - Depends on: H02, H06.
  - Verify: scripted tests for a successful body, missing `usage`, a 500, a 401, a bad JSON body and a stalled body hitting the timeout. P9 part 2: a forced error produces logs with no key and no prompt text.
  - Preserve: `reqwest` is the only HTTP path; no LLM SDK. Add `reqwest` (rustls, no default TLS feature conflict) to notices.
  - Rust lesson: `async` trait methods returning `impl Future`; why the timeout wraps the whole call.

- [ ] **H09 [S] (35 min) Quota and spend reservations on Valkey**
  - Files: `crates/server/src/hints/quota.rs`.
  - `reserve`, `settle`, `release` with the INCR/DECR logic from the design. UTC day key and expiry.
  - Requirements: R6.1–R6.8.
  - Depends on: H02, H07.
  - Verify: `valkey_` tests on a real Valkey: exactly `L` of `N ≥ L` concurrent reserves succeed (P5); spend cap refusal undoes the quota increment; `release` restores both counters; a dropped connection during reserve leaves counters consistent or returns `QuotaUnavailable`; TTL is set on first creation.
  - Preserve: counters only. No Valkey stream, list or pub/sub is used.
  - Rust lesson: `async` error handling with `?`; why each Valkey command is atomic and what that means for two clients.

- [ ] **H10 [S] (35 min) Migration `…_hints_core.sql` and privileges**
  - Files: `migrations/<timestamp>_hints_core.sql`.
  - Create `problem_hints` and `hints` exactly as in the design, using the guarded foreign key pattern for `submissions` and `users`. Grant the server role `SELECT` on `problem_hints`, and `SELECT`, `INSERT` on `hints`. Grant nothing to the judge role.
  - Requirements: R1.1, R7.1, R10.2; privileges from the design.
  - Depends on: run-lifecycle and auth migrations present (or the guarded pattern used).
  - Verify: `pg_` test: check constraints reject a ladder row with a provider, an AI row without a cache key and a familiar row with a rung; the server role cannot `UPDATE` or `DELETE` a hint row; migration runs twice without error.
  - Preserve: append-only hints; no UPDATE grant.
  - Rust lesson: not a Rust change. Skip the Rust lesson for this SQL-only task and note that in the commit message.

- [ ] **H11 [S] (35 min) Store: ladder reveal and AI rows**
  - Files: `crates/server/src/hints/store.rs`.
  - `load_submission_for_hints(user, id)` (owned and finished, else `NotFound`), `revealed_rungs`, `authored_body`, `reveal_ladder` (insert with `ON CONFLICT DO NOTHING` and read back), `find_ai(submission, rung)`, `insert_ai` (`ON CONFLICT DO NOTHING`, read back), `ghost_for(submission)`.
  - Requirements: R1.4, R1.6, R1.7, R2.4, R7.1, R7.4.
  - Depends on: H10, H03.
  - Verify: `pg_` tests: P2 (two concurrent reveals of one rung store one row and return the same text); a non-owner, unknown and unfinished submission all return `NotFound`; the ghost read returns no hidden test data (only the `ghosts` columns).
  - Preserve: every query filters by `user_id` or the submission owner; no query returns a hidden expected output.
  - Rust lesson: transactions and `ON CONFLICT` as a race-free "insert once" tool.

- [ ] **H12 [S] (40 min) AI generation pipeline `ai.rs`**
  - Files: `crates/server/src/hints/ai.rs`.
  - `generate(state, user, submission, kind, prompt)` in the order given in the design: check enabled, reserve, build prompt, run chain with timeouts, guard, cost, settle or release, return `Generated`. `AiError` covers `Disabled`, `CapReached`, `Unavailable`, `QuotaUnavailable`.
  - Requirements: R2.3, R2.6, R4, R5, R6, R7.2, R7.3, R10.1.
  - Depends on: H04, H06, H07, H08, H09, H11.
  - Verify: scripted-transport tests for: primary success (one call, settled cost, label fields set); primary failure and fallback success (two calls); both failing (release, `Unavailable`, quota restored); cap reached (no provider call); concurrent calls at the limit (P5 in the pipeline); the familiar kind uses the same quota (part of P5 test).
  - Preserve: `generate` never writes a verdict, ghost, mood or submission field (R2.6, R10.3).
  - Rust lesson: keeping one function as the only place that spends money, so every path goes through the same reserve, settle and release.

## Wave 3: HTTP surface

- [ ] **H13 [S] (40 min) Handlers and router**
  - Files: `crates/server/src/hints/handlers.rs`, `crates/server/src/hints/mod.rs`.
  - `POST /api/hints` in the order of the design: JSON and content type, `CurrentUser`, UUID and rung checks, window rate limit, ownership and finish check, gate, cache hit, code size (422 for AI), ghost fetch, `ai::generate`, insert, response. `GET /api/hints?submission_id=`. Errors use the envelope and codes from the design. `next_rung` and quota are included.
  - Requirements: R1.2–R1.7, R2.1–R2.5, R6.8, R11.1–R11.4.
  - Depends on: H03, H11, H12, H01.
  - Verify: P7 (non-owner, unknown and unfinished give identical 404 bodies, compared byte for byte); ladder reveal works with AI disabled (R6.7); AI returns 503 with `AI_DAILY_LIMIT` at zero; no `ai_*` code is returned for ladder requests.
  - Preserve: the server returns plain text bodies; no field is HTML.
  - Rust lesson: extractor order in Axum and why the cheap checks run first.

- [ ] **H14 [S] (25 min) Wire the router into the server**
  - Files: `crates/server/src/lib.rs` (add `.nest` or `.merge` for hints), `crates/server/src/hints/mod.rs`.
  - Mount under `/api`, after the health route. Pass `HintState` (config, pool, Valkey, transport) through application state.
  - Requirements: R11.3, R8.4.
  - Depends on: H13 and the `run-lifecycle` router.
  - Verify: `cargo test -p counterspell-server` builds the router; a request to `/api/hints` without a session returns `401`, and an unknown route under `/api` still returns an API error, not SPA HTML.
  - Preserve: `main.rs` stays thin.
  - Rust lesson: sharing state with `Arc`, and why state is cloned cheaply.

- [ ] **H-test [S] (30 min) Integration tests and demo reset**
  - Files: `crates/server/tests/hints_integration.rs`.
  - End-to-end `pg_` and `valkey_` tests with `ScriptedTransport`: ladder order, reveal, AI success, cache hit with no quota change, cap refusal with `next_rung`, fallback, code-too-large, and a canary check on the prompt (P12). Demo reset removes demo hints (a `pg_` test that calls `cs_reset_demo_account()` and checks the row count).
  - Requirements: R12.1, R12.2, R12.3.
  - Depends on: H13, H14, auth demo reset (from `auth`).
  - Verify: `cargo test -p counterspell-server --test hints_integration -- --ignored` with `DATABASE_URL` and `VALKEY_URL` set.
  - Preserve: tests never reach a real provider; `ReqwestTransport` is not constructed in tests.
  - Rust lesson: integration tests in `tests/` need a library target; why the server has `lib.rs`.

## Wave 4: web client

- [ ] **H15 [P] (30 min) Client module and ladder reducer**
  - Files: `web/src/lib/hints/client.ts`, `web/src/lib/hints/reducer.ts`, `web/src/lib/hints/reducer.test.ts` (if the frontend already has a unit test runner; otherwise type-checked only).
  - `requestHint({submissionId, rung, ai})` posts JSON with `credentials: 'same-origin'`. `reducer` tracks revealed rungs, AI rungs, quota and the notice flag. The client renders bodies as text nodes only.
  - Requirements: R9.1, R9.2, R2.5 (client side), R11.1.
  - Depends on: H13 API contract (no runtime dependency).
  - Verify: `cd web && npm run check`; a reducer test for "rung 3 is not requested before rung 2 is revealed" if the test runner exists.
  - Preserve: no `{@html}` for hint text; the notice text is visible next to the AI button.
  - Rust lesson: not a Rust change. Skip for this task.

## Wave 5: configuration, logs and CI

- [ ] **H16 [P] (25 min) Compose and deployment settings**
  - Files: `docker-compose.yml` (pass-through `${NAME}` entries only), `deploy/hints.md`.
  - Pass through the hint variables to `server`. Enable Valkey append-only persistence (open question 6). Document each variable and its default in `deploy/hints.md`.
  - Requirements: R8.1, R6.6 (persistence).
  - Depends on: H02.
  - Verify: `docker compose config` shows the pass-throughs and no literal key; the Valkey service has the persistence flag.
  - Preserve: no `.env` file is created, read or written. Keys are passed by the operator at deploy time.
  - Rust lesson: not a Rust change. Skip.

- [ ] **H-ci [P] (25 min) Secret and log checks in CI**
  - Files: `.github/workflows/ci.yml` (one added step), `scripts/check-no-hint-secrets.sh` (only if the repo already has a `scripts/` convention).
  - A CI step fails if a configured key pattern (`sk-…`, `gsk_…`) appears in tracked files, and the P9 log test runs as part of `cargo test`.
  - Requirements: R8.1, R8.3.
  - Depends on: H08.
  - Verify: the step fails on a deliberately placed fake key in a scratch branch (not committed to main), then passes after removal.
  - Preserve: no change to other CI jobs.
  - Rust lesson: not a Rust change. Skip.

## Wave 6: verification

- [ ] **H-verify [S] (30 min) Full checks and the manual smoke test**
  - Files: no planned changes. Fixes only if a check fails.
  - Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, `cargo deny check licenses`, `cd web && npm run check && npm run build`, `docker compose config`.
  - Manual: sign in with the demo account, reveal rung 1, request one AI hint on a failed submission, confirm the label, the notice, the quota count and that `docker compose logs server` contains no key, prompt or code.
  - Requirements: R12.4, R12.5.
  - Depends on: all tasks above.
  - Verify: every command passes; the manual checklist is recorded in `docs/kiro-log/hints-smoke.md`.
  - Preserve: no feature work beyond this spec.
  - Rust lesson: not a Rust change. Skip.

## Dependency summary

- **Wave 0:** G1 then G2. H01 in parallel with G1.
- **Wave 1:** H02, H03, H04, H05, H06, H07 all depend only on H01 (H07 also on H02). They run in parallel.
- **Wave 2:** H08 (after H02, H06), H09 (after H02, H07), H10 (after the earlier auth and run-lifecycle migrations), H11 (after H10, H03). H12 comes after H04, H06, H07, H08, H09 and H11.
- **Wave 3:** H13 after H03, H11, H12, H01. H14 after H13. H-test after H14 and the auth demo reset.
- **Wave 4:** H15 may start after the H13 API contract is written.
- **Wave 5:** H16 after H02. H-ci after H08.
- **Wave 6:** H-verify last.

## Completion checklist

- [ ] Ladder reveals are ordered, idempotent and owner-only (P1, P2, P7).
- [ ] AI hints are click-only, grounded, guarded, labelled and cached (R2, R3, R4, P3, P6, P10–P12).
- [ ] Fallback is used at most once and only after a failure (P4).
- [ ] Quota and spend are exact under concurrency and released on infrastructure failure (P5, R6).
- [ ] Cost is recorded with provider, model, tokens and estimate flag (P8).
- [ ] No key, prompt or code appears in logs or errors (P9).
- [ ] The familiar uses the same client and quota and cannot change its mood (R10).
- [ ] Open questions 1–18 in `design.md` are answered, or the assumed choice is accepted.
