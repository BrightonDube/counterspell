# Review: run-lifecycle recent commits — 14 Nov 2026

Reviewer: Devin (read-only). Scope: commits `bb612e5`..`65c5564` on
`brighton/kiro-setup` — run-lifecycle T27, T28, T32, T33, T38 plus the attempt-7
blocker note; auth attempt-5 and ide-workspace attempt-6 blocker notes.

## Verification results (this session, WSL, throwaway Postgres 17 + Valkey 8)

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --workspace --all-targets -- -D warnings` | PASS |
| `cargo test --workspace` (no DB) | PASS — 62 test binaries, 0 failures; `pg_`/`valkey_` ignored |
| `cargo test --workspace -- --include-ignored` (with DB) | PASS on re-run — 62 binaries ok, 0 failures. First run had one failure (`pg_model_interleavings`, `VersionMissing`) caused by a **stale `sqlx::migrate!` embed**: the test binary predated the auth migrations and cargo's fingerprinting on `/mnt/c` (drvfs) did not rebuild it. `cargo clean -p counterspell-judge` + rebuild → passes in 8.17 s. Environmental, not a code bug — but worth remembering: any `pg_` failure mentioning `VersionMissing` on this machine means `cargo clean -p <crate>` first. |
| `cargo deny check licenses` | PASS (`licenses ok`; two benign unmatched-allowance warnings: ISC, MIT-0) |
| `gitleaks detect` | PASS — no leaks in 228 commits |
| `web`: `npm run lint` | PASS (no output) |
| `web`: `npm run check` (svelte-check) | PASS — 0 errors, 0 warnings |
| `web`: `npm test` (vitest) | PASS — 36 files, 243 tests |
| `web`: `npm run build` | PASS |

DB-backed result: full `--include-ignored` suite re-run after the judge rebuild — 62 binaries ok, 0 failures (see table above).

## Commit-by-commit

### `bb612e5` T27 — cancel running + single-terminal tests — APPROVE

Adds `pg_cancel_running_stops_the_executor_and_finishes_as_cancelled` and
`pg_cancel_never_yields_both_done_and_error` to `crates/server/tests/cancel.rs`.
Both are real `#[sqlx::test]` integration tests against a live worker with a
`ScriptedExecutor` (`Hang`, then a slow `Pass`). Asserts the judge stops within
a deadline, the run finishes `failed`/`cancelled` with no verdict, and exactly
one terminal event exists — the race is genuinely exercised (cancel lands while
claimed, 40×50 ms script). Meaningful tests, not tautologies. Task ticked
correctly per spec (T27 requires those exact `pg_` tests).

### `3c8a090` T28 — RunWaker — APPROVE

`crates/server/src/runs/wake.rs`: one `PgListener` on `run_events`, broadcast
cap 1024, `Wake::Run(RunId)` carries only a run id (never event data), and
`Wake::All` is sent after every (re)connect so streams re-read from Postgres —
the correct response to notifications lost during a gap. Reconnect loop with a
1 s delay; payloads that aren't run ids are ignored. Matches R12.5/R13.3/R13.4
and the spec's "waker is a hint only" preserve-note. `WakerMode::Disabled` /
`idle()` gives the poll-only path for tests.

Minor observation (non-blocking): on `Ok(None)` from `try_recv` the loop sends
`Wake::All` then immediately retries; if sqlx returns `Ok(None)` repeatedly
while reconnecting, this could spin briefly. Harmless — `Wake::All` only makes
streams re-read — and `pg_waker_reconnect_sends_wake_all` covers the behaviour.
Verify once against the live DB (running now).

### `2d58b7a` T32 — reducer P10 property + transient-drop stream test — APPROVE

200 fixed seeds of a small LCG inject duplicates/replays into a 5-event log;
final state must equal the single-apply state. Deterministic, actually exercises
the reducer's de-dup/ordering. The `stream.test.ts` addition covers a drop while
`readyState = CONNECTING` — the browser's own retry keeps the same source, no
reopen. Matches spec (P10 seeded property, transient-drop case).

### `a4a6e3e` T33 — idempotency key per user action — APPROVE

`startRunAction` mints one `crypto.randomUUID()` key per action; `send` reuses
it on every retry. Exactly the spec's "one Idempotency-Key per user action"
requirement. Tests cover key reuse after a 500 and a fresh key per action.

### `4e9d4a2` T38 — Caddy SSE route, per-role DB URLs, deploy notes — APPROVE

- `Caddyfile`: `@run_events path_regexp ^/api/runs/[^/]+/events$` gets its own
  `handle` with `reverse_proxy … { flush_interval -1 }` and sits **before** the
  default `handle` that applies `encode zstd gzip` — so SSE is unbuffered and
  uncompressed while everything else keeps compression. Correct Caddy idiom.
- `docker-compose.yml`: `DATABASE_URL: ${SERVER_DATABASE_URL:-}` /
  `${JUDGE_DATABASE_URL:-}` — names only, no values; empty default fails closed
  at server/judge config validation rather than silently using a wrong DB.
- `deploy/run-lifecycle.md`: role creation, config table, single-server-process
  caveat (per-user stream limits are per-process), Caddy check, DoD report
  instructions. Accurate and appropriately scoped.
- Honestly deferred: the live "SSE through Caddy arrives as written" check
  needs a running stack — recorded as deferred in the blocker notes, which is
  the right call for a no-E2E pass.

Minor observation (non-blocking): the path regexp won't match a trailing
slash (`/api/runs/x/events/`) — that request would fall through to the
compressed handle and buffer. The client uses the exact path, so this is only
a robustness note, not a bug today.

### Blocker notes (`9621eda`, `dfc9b2a`, `1890f86`, log commits) — accurate

Attempt-7 (run-lifecycle), attempt-5 (auth) and attempt-6 (ide-workspace) notes
correctly identify the real blockers and don't claim unverified progress.
T29's `futures-util` analysis is correct: axum 0.8.9 does not re-export a
stream trait, so `Sse`/`Body::from_stream` need a direct dependency no task
names — a legitimate Brighton decision, not Kiro timidity.

## Findings

### From the primary review

No blocking findings in the commits themselves. Two minor observations
(waker reconnect spin — harmless and covered by `pg_waker_reconnect_sends_wake_all`;
Caddy trailing-slash edge — cosmetic). The work is test-first, commits are
atomic per task, and `tasks.md` ticks match delivered scope.

### From the independent second reviewer (verified by driver)

**B1 — BLOCKER, verified: `docker-compose.yml` server service lacks `VALKEY_URL`.**
`ServerConfig::from_lookup` requires it (`crates/server/src/config.rs:103`) and
`main.rs:42` opens `ValkeyLimiter` from it, but the compose `server.environment`
block sets only `HOST`, `PORT`, `STATIC_DIR`, `DATABASE_URL`. `docker compose
config` passes (it validates YAML only), so T38's verify missed it — the
container exits with `ConfigError::Missing("VALKEY_URL")` and restart-loops.
`deploy/run-lifecycle.md` even lists `VALKEY_URL` in its config table while the
compose file never wires it. Affects the prod path too if `compose.prod.yml`
doesn't inject it. → fix prompt sent to Kiro.

**M1 — MAJOR, verified, needs Brighton (pre-existing, pre-production):**
`docker-compose.yml:61` sets `POSTGRES_HOST_AUTH_METHOD: trust`. Any container
on the `internal` network can connect to Postgres as any role — including the
`counterspell` superuser used by `migrate` — with no password, which voids the
R19 least-privilege story the per-role `DATABASE_URL` split is meant to provide.
Not introduced by this batch (T38's scope was server/judge env only) and deploy
is not live yet, but it must be resolved (scram password via env) before the
first real deploy.

**M2 — MAJOR, verified: `web/src/lib/runs/reducer.ts:122` mis-maps error codes.**
`payload.code === 'internal_error'` is never true — `internal_error` is a
`RunStatus`, not a `FailureReason`. Real codes (`status.rs:270-278`) are
`cancelled`, `problem_unavailable`, `language_unavailable`, `expired`,
`retries_exhausted`. Rust `run_state.rs` maps `Expired→expired`,
`RetriesExhausted→internal_error`, rest→`failed`; the reducer maps everything
to `failed`. `reducer.test.ts:125,132` "verifies" with the impossible code
`'internal_error'` — vacuous for real codes; no test covers `expired` or
`retries_exhausted`. Bounded impact (R14.4 refetches `GET /api/runs/{id}` after
terminal) but `RunStreamState.status` is wrong in the interim. → fix prompt sent
to Kiro.

**Minors (tracked, no action yet):**
- `RunWaker::start` is never called — `AppState` defaults to `RunWaker::idle()`
  so production SSE is poll-only (correct per R13.4, but T29/T30 or deploy work
  must wire it or the waker is dead code).
- `stream.ts`: `reconnects` counter never resets on healthy flow — a long-lived
  watch dies after 5 lifetime gaps even if all succeed.
- `cancel.rs`: `pg_notify` runs after the `UPDATE` autocommits; a notify failure
  returns 500 although the flag was set (retry is harmless — `COALESCE` keeps
  the first timestamp).
- `docker-compose.yml`: `migrate` has no `depends_on: service_completed_successfully`
  ordering before `server`/`judge` — first `up` races migrations (self-heals on
  restart).
- T28 spec deviation (`WakerMode::Disabled` → `RunWaker::idle()`): functionally
  equivalent, honestly recorded in `run-lifecycle-blockers.md`.
- Nits: no unauthenticated cancel test case; `restarted` sets `status:'queued'`
  while the run is actually `claimed`; auth/content-type check ordering differs
  between `cancel.rs` and `create.rs`.

## State of the specs (checkbox counts, 14 Nov)

| Spec | Done/Total | Status |
|---|---|---|
| ghost-engine | 37/37 | complete |
| grimoire-packs | 26/30 | T20 needs judge-sandbox; T28 needs Brighton's pack review; T29 stale-blocked (ghost-engine now done — see report); T30 needs CI owner |
| run-lifecycle | 31/56 | **blocked at T29** on `futures-util`/`futures-core` approval; T30+ chain behind it |
| auth | 20/55 | T01 licence decision (`reqwest`/`oauth2` → `webpki-roots` CDLA-Permissive-2.0) + DB now available |
| judge-sandbox | 0/56 | gated on T00 — review prepared at `docs/reviews/judge-sandbox-T00.md` |
| language-drivers | 0/27 | gated on judge-sandbox ≥ half done |
| ide-workspace | 13/39 | upstream-blocked (catalogue tables, SSE route, fonts, hints/familiar routes); rest deferred E2E |
