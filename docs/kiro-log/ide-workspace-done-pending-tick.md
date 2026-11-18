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
