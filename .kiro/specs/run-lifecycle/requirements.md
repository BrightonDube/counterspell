# Requirements: run-lifecycle

## Introduction

A **run** is one Run or Submit request from the IDE. This spec defines how a run travels from `POST /api/runs` to a final, fetchable result: how it is saved, queued, claimed by a worker, recovered after a crash, streamed live to the browser, and finished exactly once.

**PostgreSQL is the only authoritative store** for submissions, the job queue and run events. Valkey is used only for rate-limit counters (and later caches). It is never the queue and never the event bus. Postgres `LISTEN`/`NOTIFY` is used only as a wake-up hint; every event is first a durable row.

The work is split along the trust boundary from `docs/system-design.md` §2:

- `counterspell-server` accepts requests, enforces validation, rate limits and ownership, and serves reads and SSE. It never executes code, never writes run events, and never writes a verdict.
- `counterspell-judge` (trusted worker) claims jobs, holds leases, writes every run event, and writes the one terminal result. The code that runs a program inside the sandbox is **not** part of this spec. This spec defines the seam it plugs into (`RunExecutor`).
- `counterspell-common` holds the shared run state machine, verdict and event types.

Property-based testing (PBT) is **enabled** for this spec. The correctness properties are in `design.md` and map back to the requirements below.

Sources: `docs/system-design.md` §2, §4 (`jobs`, `run_events`, `submissions`), §5 (Run lifecycle), §7.1, §9; `docs/requirements.md` IDE-3, VAL-1 to VAL-5, and the Sandbox criteria; steering files `api-conventions.md`, `security.md`, `sandbox.md`, `structure.md`, `rust-teaching.md`.

## Glossary

- **Run / submission**: one Run or Submit request and its stored row in `submissions`. The two words mean the same thing here.
- **Job**: the queue row in `jobs` for a run. One job per run (`jobs.submission_id` is unique).
- **Attempt**: one try by one worker to execute a job. Attempts are numbered from 1 in events and from 0 in `jobs.attempts` (the count of attempts that were abandoned).
- **Worker**: one `counterspell-judge` process. It has a unique `worker_id` generated at start-up.
- **Lease**: the time-limited exclusive right of one worker to execute one job. Stored as `lease_owner` and `lease_expires_at`, computed with the database clock.
- **Fence**: the triple (job id, `lease_owner`, `attempts`). A write for a run is accepted only while the job row still shows that exact triple as `claimed`.
- **Reaper**: the periodic routine in each judge process that recovers expired leases, expires stale queued jobs and finishes cancelled work.
- **Executor**: the component that actually runs a claimed run (later: the sandbox runner). Here it is a trait with a fake implementation for tests.
- **Terminal state**: `completed`, `failed`, `expired` or `internal_error`. A terminal run never changes again.
- **Public event**: a row in `run_events`. Its payload contains only data the owner may see.
- **Wake-up hint**: a `NOTIFY` that says "something changed, look in the table". It carries no event data and is never trusted for correctness.
- **Cursor**: the highest `seq` a client has been sent (the SSE `Last-Event-ID`).
- **Canary**: a unique marker string placed in hidden test data during tests to prove it never leaks.

## Scope constraints

- The Cargo workspace keeps exactly the six canonical packages from `repo-skeleton`. This spec adds **no** package, crate or parallel layout. New code goes into `crates/common/`, `crates/server/`, `crates/judge/`, `migrations/`, `web/src/lib/runs/`, and the CI, Compose and Caddy files.
- Postgres is authoritative. Valkey Streams, Valkey pub/sub and any in-memory queue SHALL NOT carry jobs or events.
- The server SHALL NOT depend on `bollard` and SHALL NOT have access to any Docker daemon.
- Running user code, language drivers, the comparator, hidden test storage, ghost generation, bench fitting, hints, authentication, the problem catalogue, the dashboard and the full UI are **other specs**. This spec defines the seams to them and tests with a fake executor.
- The SSE event names stay exactly as in `.kiro/steering/api-conventions.md`: `compiling`, `test`, `ghost_hunting`, `shrink_step`, `ghost`, `bench_point`, `complexity`, `familiar`, `restarted`, `done`, `error`.
- Numbers marked **[E]** are estimates. They are configuration defaults, to be tuned after the VPS load test.

## Requirements

### R1. Authoritative storage and boundaries

**User story:** As the operator, I want one source of truth for runs, so that no run can be lost or duplicated by a cache, a broker or a restart.

1. THE SYSTEM SHALL store submissions, jobs, run events and test results only in Postgres, and SHALL derive every user-visible run fact from those rows.
2. THE SYSTEM SHALL use Valkey only for rate-limit counters. Valkey keys SHALL hold counters only, never code, events, verdicts or job data.
3. THE SYSTEM SHALL NOT use Valkey Streams, Valkey pub/sub, or an in-process channel as a job queue or as the source of events.
4. THE judge SHALL be the only component that writes `run_events`, `test_results`, the lifecycle `status` of a submission after enqueue, and the terminal result.
5. THE server SHALL only insert submissions and jobs, set a job's cancel flag, and read.
6. THE judge SHALL need no Valkey URL and no secret other than its limited Postgres credentials and the sandbox socket (the latter is added by the judge-sandbox spec).

### R2. Atomic enqueue

**User story:** As a learner, I want a run I submitted to be queued for sure, so that I never wait for a run that does not exist.

1. WHEN `POST /api/runs` has passed authentication, validation and the limit checks, THE server SHALL insert the submission (`status = queued`) and its job (`state = queued`, `attempts = 0`) in **one Postgres transaction**.
2. THE server SHALL answer `202 {run_id, status}` only after that transaction has committed.
3. IF any statement in that transaction fails, THEN neither the submission nor the job SHALL exist afterwards, and the response SHALL be a JSON error that reveals no internals.
4. THE database SHALL enforce `UNIQUE (jobs.submission_id)` so that a run can never have two jobs.
5. THE enqueue transaction SHALL send a `NOTIFY run_jobs` (wake-up only) so that an idle worker claims the job without waiting for its next poll.
6. THE server SHALL return the run id within 300 ms at p95, excluding network time (IDE-3.1). The test harness SHALL measure and report this separately from completion latency.

### R3. Idempotency-Key

**User story:** As a learner on a flaky connection, I want a retried request to return my original run, so that I do not get two runs and lose quota.

1. THE server SHALL accept an optional `Idempotency-Key` header of 1 to 128 characters from `[A-Za-z0-9._-]`. IF the value is invalid, THEN THE server SHALL respond `400 invalid_idempotency_key` and create nothing.
2. WHEN a request carries a key that the same user already used with the **same** problem, language, mode and code hash, THE server SHALL return `202` with the original `run_id`, set `Idempotent-Replayed: true`, and create no submission, no job and no new event.
3. IF a key was already used by the same user with different problem, language, mode or code, THEN THE server SHALL respond `422 idempotency_key_reuse` and create nothing.
4. THE database SHALL enforce uniqueness of `(user_id, idempotency_key)` for rows where the key is not null. Keys of different users SHALL NOT collide.
5. WHEN several requests with the same user and key arrive concurrently, THE system SHALL create exactly one submission and one job, and every request SHALL receive the same `run_id`.
6. THE server SHALL NOT count an idempotent replay against the Run/Submit rate limits or the queued-jobs cap.
7. WHERE no key is sent, THE server SHALL treat each request as a new run.

### R4. Submission validation at the endpoint (VAL-1, VAL-2, VAL-3)

**User story:** As the operator, I want every check repeated on the server, so that skipping the browser checks gains an attacker nothing.

1. THE server SHALL accept only `Content-Type: application/json` with one object `{problem, lang, code, mode}`. Unknown fields SHALL be rejected. The HTTP layer SHALL cap the request body (default 128 KiB) before parsing.
2. THE server SHALL accept `mode` values `run` and `submit` only, and `lang` values that are enabled in the server's language list. IF `problem` does not name a published problem, THEN THE server SHALL respond `404 unknown_problem`.
3. IF the code is larger than the per-language byte limit (default 65 536 bytes), THEN THE server SHALL respond `413 source_too_large` with a message that states the limit and the actual size, before anything is queued. THE limit is in bytes, not lines (VAL-1).
4. IF the code contains a NUL byte or a control character other than tab, newline or carriage return, THEN THE server SHALL respond `400 invalid_source` before queuing (VAL-3).
5. THE server SHALL parse the code with tree-sitter using the grammar of the selected language. IF the syntax tree contains an error, THEN THE server SHALL respond `400 source_does_not_parse` with the line and column of the first error and SHALL NOT queue the run (VAL-2).
6. THE server SHALL NOT use keyword or API blocklists as a security control.
7. A request rejected by validation SHALL create no submission, no job and no event, and SHALL NOT use up Run/Submit quota.

### R5. Rate limits and queue caps (VAL-4, system design §7.1)

**User story:** As the operator, I want one person or IP unable to starve the queue, so that every learner gets their turn.

1. THE server SHALL enforce a per-user and a per-IP limit on `POST /api/runs` using Valkey counters in fixed windows. Defaults **[E]**: 30 runs per 600 s per user, and 100 runs per 600 s per IP.
2. IF a window limit is exceeded, THEN THE server SHALL respond `429 rate_limited` with a `Retry-After` header and a `retry_after_seconds` field of at least 1, and SHALL NOT queue the run.
3. THE server SHALL enforce a maximum number of **active** jobs per user, counted from the Postgres `jobs` table (`state` is `queued` or `claimed`). Default **[E]**: 3. The count and the insert SHALL happen in the same transaction under a per-user lock, so concurrent requests cannot exceed the cap.
4. IF the cap is reached, THEN THE server SHALL respond `429 queue_cap` with a wait estimate of at least 1 s.
5. THE server SHALL enforce a global queue depth cap (default **[E]** 200 queued jobs) and respond `429 queue_full` when it is reached (system design §7, "Abuse").
6. IF Valkey is unavailable, THEN THE server SHALL log the failure, skip the Valkey windows, and still enforce the Postgres caps (fail open for the windows, closed for the caps).
7. THE server SHALL read the client IP from the proxy header only when the connection comes from a configured trusted proxy. Otherwise it SHALL use the socket address.
8. THE Valkey keys SHALL be namespaced (`rl:runs:user:<id>`, `rl:runs:ip:<hash>`), expire with their window, and hold nothing but a counter.

### R6. Run state machine

**User story:** As a developer, I want a single, explicit state machine, so that every part of the system agrees on what a run can be doing and what can never happen.

1. THE SYSTEM SHALL define the run `status` values: `queued`, `claimed`, `compiling`, `running`, `ghost_hunting`, `completed`, `failed`, `expired`, `internal_error`.
2. THE SYSTEM SHALL treat `completed`, `failed`, `expired` and `internal_error` as terminal. A terminal run SHALL NOT change status, verdict, counts, score or `finished_at` again.
3. THE SYSTEM SHALL allow only the transitions listed in `design.md` ("Transition table"). Any other transition SHALL be rejected by the Rust state machine **and** by a database trigger.
4. `completed` SHALL carry a `verdict` (`accepted`, `wrong_answer`, `runtime_error`, `time_limit`, `memory_limit`, `output_limit`, `compile_error`). No other status SHALL carry a verdict.
5. `failed`, `expired` and `internal_error` SHALL carry a `failure_reason`. `completed` SHALL NOT.
6. THE meaning of the terminal states SHALL be: `completed` = the trusted judge produced a verdict for the user's code; `failed` = the run ended without a verdict for a deterministic, non-infrastructure reason (`cancelled`, `problem_unavailable`, `language_unavailable`); `expired` = the run waited in the queue past its time-to-live and was never executed; `internal_error` = infrastructure failed or retries were exhausted.
7. THE database SHALL keep `jobs.state` consistent with `submissions.status` according to the mapping in `design.md`, and a terminal submission SHALL have `finished_at` set while a non-terminal one SHALL NOT.
8. THE shared Rust state machine in `counterspell-common` SHALL be the single definition used by the server and the judge.

### R7. Exclusive claim

**User story:** As the operator, I want each job executed by one worker at a time, so that a run is never judged twice at once.

1. A worker SHALL claim a job with one statement that selects the next eligible job with `FOR UPDATE SKIP LOCKED LIMIT 1` and marks it `claimed` with `lease_owner` and `lease_expires_at = now() + lease`, using the database clock.
2. A job SHALL be eligible only when `state = 'queued'` and `run_after <= now()`. Jobs with a cancel request SHALL be claimed first so they finish quickly. Otherwise the order SHALL be `run_after`, then `id`.
3. THE claim SHALL, in the same transaction, move the submission to `claimed`, set `started_at` if it is still null, and, WHEN `attempts > 0`, append the `restarted` event.
4. WHILE a job is `claimed` with a live or unreaped lease, no other worker SHALL be able to claim it.
5. A worker SHALL claim only while it has a free execution slot. The number of slots SHALL be `JUDGE_CONCURRENCY` (default 2).
6. WHEN idle, a worker SHALL wake on `NOTIFY run_jobs` and SHALL also poll at a short interval (default **[E]** 1 s), so a lost notification only delays a claim.
7. A worker SHALL NOT claim a job it already holds in another slot.

### R8. Renewable lease

**User story:** As the operator, I want a worker that is alive to keep its job and a worker that is gone to lose it.

1. WHILE a worker executes a job, it SHALL renew the lease every `lease / 3` (defaults: lease 30 s, renew every 10 s **[E]**), using a fenced update that extends `lease_expires_at` and returns the cancel flag.
2. IF a renewal updates no row, THEN the worker SHALL treat the lease as lost: it SHALL signal the executor to stop (killing any sandbox), and SHALL NOT write anything more for that claim.
3. IF the worker cannot renew for a full lease period because the database is unreachable, THEN the worker SHALL consider the lease lost locally and SHALL stop the executor, even though it cannot confirm the loss.
4. THE fence SHALL include `attempts`, so a worker that loses a lease and later re-claims the same job gets a new fence, and a stale task from the older claim cannot write.
5. A worker SHALL NOT take the lease time from its own clock. Expiry decisions SHALL use the database clock only.

### R9. Crash recovery and expiry

**User story:** As a learner, I want a run to finish even if a worker dies, so that I am never left looking at a spinner forever.

1. EVERY judge process SHALL run the reaper at start-up and then periodically (default **[E]** every 5 s). Several reapers at once SHALL be safe.
2. THE reaper SHALL find claimed jobs whose `lease_expires_at` is in the past, locking them with `FOR UPDATE SKIP LOCKED`.
3. FOR each such job with `attempts + 1 < max_attempts`, THE reaper SHALL, in one transaction, set the job to `queued` with `attempts = attempts + 1`, a backoff `run_after`, and no lease, and set the submission to `queued`.
4. FOR each such job with `attempts + 1 >= max_attempts`, THE reaper SHALL, in one transaction, set `attempts = attempts + 1`, the job `dead`, the submission `internal_error` with `failure_reason = retries_exhausted`, `finished_at`, and append one `error` event with a safe message.
5. IF a reaped job has a cancel request, THEN THE reaper SHALL finish it as `failed` with reason `cancelled` instead of requeueing it.
6. THE reaper SHALL expire jobs that have been `queued` longer than the queue time-to-live (default **[E]** 600 s): job `failed`, submission `expired`, one `error` event with code `expired`. An expired run SHALL never be executed.
7. A run SHALL NOT remain in `claimed`, `compiling`, `running` or `ghost_hunting` longer than lease + reaper interval + a small tolerance after its worker stopped renewing. When it leaves that state it SHALL be requeued or terminal.
8. `jobs.attempts` SHALL never exceed `jobs.max_attempts` (default 3).
9. WHEN a worker receives a shutdown signal, it SHALL stop claiming, give running work a grace period, and then release unfinished jobs through the same requeue path as the reaper.
10. A live heartbeat is not proof of progress. THE worker SHALL stop renewing, and THE reaper SHALL treat the lease as expired, once the time since `claimed_at` exceeds the maximum claim age (default **[E]** 120 s; the sandbox's own hard cap is 20 s). A live but stuck executor SHALL therefore not hold a run forever.

### R10. Fenced writes and exactly-once terminal write

**User story:** As a learner, I want exactly one final answer for my run, so that a retry can never flip my result.

1. EVERY judge write that belongs to a claimed run (phase change, event, auxiliary row, terminal result) SHALL happen in a transaction that first locks the job row and confirms `state = 'claimed' AND lease_owner = me AND attempts = my attempts`. IF that check finds no row, THEN the write SHALL do nothing and report `LeaseLost`.
2. THE terminal write SHALL be **one transaction** that: inserts the `test_results`, appends the terminal event (`done` or `error`), sets the submission's final status, verdict, counts and `finished_at`, and sets the job to its terminal state, all guarded by the fence of R10.1.
3. A second terminal write for the same run SHALL be impossible, by three independent barriers: the fence, a trigger that freezes a terminal submission, and a unique index that allows only one terminal event per run.
4. `test_results` SHALL be written only in the terminal transaction, and SHALL have the primary key `(submission_id, ordinal)`. Rows from an abandoned attempt SHALL NOT exist.
5. Auxiliary rows written before the terminal transaction (for example ghosts or timings, owned by other specs) SHALL be written through the same fence and SHALL be idempotent (upsert or replace).
6. A deterministic executor SHALL produce the same persisted final result whether the run finished on the first attempt or after any number of abandoned attempts.
7. A worker that lost its lease SHALL NOT be able to change the job, the submission, the events or the results, even if its executor keeps running for a short time.

### R11. Persisted, ordered run events

**User story:** As a learner, I want every step of my run recorded in order, so that I can reload the page or lose my connection and still see what happened.

1. EVERY public event SHALL be inserted into `run_events` before it is announced anywhere.
2. THE judge SHALL assign `seq` inside the same transaction as the insert, by incrementing a counter on the submission row. `seq` SHALL start at 1 and increase by exactly 1 per event.
3. Because the counter row is locked until commit, `seq` order SHALL equal commit order. No reader SHALL ever see event `n + 1` committed while event `n` is still invisible.
4. WHEN a worker claims a job whose `attempts > 0`, THE first event of that attempt SHALL be `restarted {attempt}`, committed together with the claim.
5. THE `type` column SHALL be restricted by a database CHECK to the eleven allowed event names. THE payload SHALL be a JSON object of at most 16 KiB.
6. A run SHALL have exactly one terminal event (`done` or `error`), and it SHALL be the event with the highest `seq`. No event SHALL be appended after it.
7. THE writer SHALL send `NOTIFY run_events` with the run id as the only payload, in the same transaction.
8. A run's `status` SHALL change only in the same transaction as the cause of the change: the claim, the `compiling` event, the first `test` event, the `ghost_hunting` event, the terminal write, or the reaper or release requeue. The current status SHALL always be readable from `GET /api/runs/{id}`, because the event names do not include `queued` or `running`.

### R12. SSE authorization

**User story:** As a learner, I want nobody else to see my code's results, and I want the server to prove it before sending a single byte.

1. `GET /api/runs/{id}/events` SHALL respond `401` when there is no valid session.
2. WHEN the signed-in user does not own the run, or the run does not exist, THE server SHALL respond with the **same** `404 not_found` body, so that run ids cannot be probed.
3. THE ownership check SHALL complete before the server runs any query for events and before it opens the stream.
4. EVERY event query SHALL itself contain the owner predicate (a join on `submissions.user_id = $user`), so that no code path can return events without a user.
5. THE wake-up notification SHALL contain only the run id and SHALL be filtered by the stream's own run id.
6. THE server SHALL limit concurrent streams per user (default **[E]** 5) and respond `429 too_many_streams` beyond that.
7. THE server SHALL close any stream after a maximum age (default **[E]** 900 s). A client may reconnect with `Last-Event-ID` and loses nothing.

### R13. SSE replay and wake-up

**User story:** As a learner, I want a dropped connection to resume exactly where it left off.

1. EVERY SSE message SHALL use `id: <seq>`, `event: <type>` and `data: {"seq":…, "attempt":…, "payload":{…}}`.
2. THE stream SHALL start from the cursor given by the `Last-Event-ID` header (or a `last_event_id` query parameter for non-browser clients), or from 0 if neither is present. IF the value is not a non-negative integer, THEN THE server SHALL respond `400 invalid_last_event_id`.
3. THE server SHALL subscribe to wake-ups **before** its first event query, then replay every event with `seq > cursor` in order from Postgres, in batches.
4. AFTER the replay, THE server SHALL wait for a wake-up hint or the poll fallback (default **[E]** 250 ms), and then query again from the cursor. Correctness SHALL NOT depend on any `NOTIFY` arriving.
5. THE server SHALL hold no database connection while it waits.
6. WHEN the terminal event has been sent, THE server SHALL close the stream. WHEN a client connects with a cursor at or beyond the terminal `seq`, THE server SHALL respond `204 No Content` so the browser stops reconnecting.
7. THE server SHALL send a comment heartbeat every 15 s and the headers `Cache-Control: no-store`, `X-Accel-Buffering: no` and `X-Content-Type-Options: nosniff`.
8. A new event SHALL reach a connected client within 1 s of its commit, with notifications working or disabled (IDE-3.2).
9. THE server SHALL keep no authoritative in-memory run state, so a server restart loses nothing; clients reconnect and replay.

### R14. Client de-duplication

**User story:** As a learner, I want my test list to stay correct when events are repeated or the connection restarts.

1. THE browser client SHALL track `lastSeq` and SHALL ignore any event whose `seq` is less than or equal to it.
2. THE client SHALL apply events in `seq` order. IF it receives `seq > lastSeq + 1`, THEN it SHALL treat that as a gap, close the stream, and reconnect with `last_event_id = lastSeq`.
3. WHEN the client receives `restarted`, THE client SHALL clear the per-test rows (and any ghost, bench and familiar state from the earlier attempt) and keep `lastSeq`.
4. WHEN the client receives the terminal event, THE client SHALL close the stream (so the browser does not reconnect) and SHALL then fetch `GET /api/runs/{id}` to display the authoritative result.
5. THE event reducer SHALL be a pure function with unit and randomised tests, and SHALL render server text as plain text only.

### R15. Public-only payloads

**User story:** As the operator, I want hidden tests to stay hidden, so that nobody can read the answers out of the stream.

1. THE judge SHALL build event payloads only from typed public payload structs defined in `counterspell-common`. There SHALL be no function that takes a raw JSON value from the executor and appends it.
2. A `test` event for a hidden test SHALL contain `ordinal`, `verdict` and `time_ms` only. A `test` event for a sample test MAY also contain the sample's input, expected and actual output (IDE-2.2).
3. Public events, `test_results` and `GET /api/runs/{id}` SHALL NOT contain any hidden test input, hidden expected output, or any stdout or stderr text produced by the user's program for a hidden test, because that program receives the hidden input and could echo it.
4. THE database SHALL enforce this for stored results: a `test_results` row that is not a sample SHALL have null `public_detail`, `stdout_snip` and `stderr_snip` (CHECK constraint).
5. `error` payloads SHALL carry a stable `code` and a safe `message`, and SHALL NOT contain stack traces, file paths, SQL, worker ids, secrets or hidden data.
6. A `done` payload for a `compile_error` verdict MAY include the compiler output for the user's own code, truncated to 8 KiB.
7. THE server SHALL serve run data as `application/json` or `text/event-stream` with `nosniff`, and the client SHALL render all program text as plain text (VAL-5).
8. A canary test SHALL place unique markers in hidden inputs and expected outputs, run the whole pipeline, and fail if any marker appears in any `run_events` row, `test_results` row, `GET` response or SSE byte.

### R16. One authoritative final result

**User story:** As a learner, I want my result to be there when I reload the page.

1. `GET /api/runs/{id}` SHALL return `401` without a session and the same `404 not_found` for a missing or foreign run.
2. THE response SHALL contain at least: `run_id`, `problem`, `lang`, `mode`, `status`, `terminal`, `verdict`, `failure_reason`, `attempt`, `passed`, `total`, `score`, `runtime_ms`, `memory_kb`, `created_at`, `started_at`, `finished_at`, `last_seq` and the per-test results (public detail only).
3. WHEN the run is terminal, THE response SHALL be built only from stored final columns and SHALL be identical on every later request.
4. THE `done` event payload and the `GET` response SHALL agree on `status`, `verdict`, `passed`, `total`, `score`, `runtime_ms` and `memory_kb`, because both are written from the same final-result value in the same transaction.
5. WHEN the run is not terminal, THE response SHALL show the current status and `terminal: false`.
6. Sections owned by other specs (ghost, complexity, familiar) SHALL be added to the same read model by those specs. This spec SHALL leave them absent, never fabricated.

### R17. Cancellation

**User story:** As a learner, I want to stop a run I no longer want.

1. `POST /api/runs/{id}/cancel` SHALL require a session and ownership (`404` otherwise). IF the run is queued or active, THEN it SHALL set `jobs.cancel_requested_at` (once), send `NOTIFY run_cancel` and respond `202 {status}`. IF the run is terminal, THEN it SHALL change nothing and respond `200` with the terminal status.
2. Repeated cancel requests SHALL be harmless.
3. WHEN a **queued** job is cancelled, THE judge SHALL claim it ahead of other jobs and finish it as `failed` with reason `cancelled`, without invoking the executor.
4. WHEN a **claimed** job is cancelled, THE worker SHALL learn of it from `NOTIFY run_cancel` or the next lease renewal (at most one renewal interval), stop the executor, kill the sandbox, and finish the run as `failed` with reason `cancelled`. A cancelled run SHALL NOT be retried.
5. IF the executor has already produced a complete verdict when the cancellation is seen, THEN THE terminal write SHALL keep that verdict (`completed`). A run SHALL never end with both a `done` and an `error` event.
6. A cancelled run SHALL end with one `error` event with code `cancelled`.
7. A cancel request SHALL NOT change any other user's run, and SHALL NOT consume Run/Submit quota.

### R18. Infrastructure failure is never a verdict

**User story:** As a learner, I want a broken sandbox to be called a broken sandbox, not a wrong answer.

1. IF the executor reports an infrastructure error (sandbox daemon down, image missing, worker task panic), THEN THE judge SHALL release the job for retry with backoff, counting an attempt, and SHALL NOT write a verdict.
2. WHEN the retries are exhausted, THE run SHALL end as `internal_error` with one `error` event and a safe message.
3. IF the executor reports a deterministic non-retryable condition (`problem_unavailable`, `language_unavailable`), THEN THE run SHALL end as `failed` with that reason, without retries.
4. A panic or abort inside an executor task SHALL be caught by the worker and treated as an infrastructure error. It SHALL NOT stop the worker process or other slots.

### R19. Least-privilege database access

**User story:** As the operator, I want a compromised web server to be unable to forge a verdict.

1. THE migrations SHALL create two non-login roles, one for the server and one for the judge, with exactly the privileges in the privilege matrix in `design.md`. Deployment grants login users membership in these roles.
2. THE server role SHALL NOT be able to insert into `run_events` or `test_results`, update `submissions`, or change any column of `jobs` except `cancel_requested_at` and `updated_at`.
3. THE judge role SHALL NOT be able to delete rows from the run tables, and SHALL NOT be able to change `user_id`, `code` or `code_sha256` of a submission.
4. A test SHALL assume each role (`SET ROLE`) and prove each denied action fails and each required action works.

### R20. Observability and latency measurement

**User story:** As Brighton, I want measured numbers, not guesses.

1. THE server and judge SHALL emit structured `tracing` events with `run_id`, `attempt`, `worker_id` and durations. They SHALL NOT log source code, hidden data, or secrets.
2. THE test harness SHALL measure **enqueue latency** (client sends `POST` until the `202` arrives) and **completion latency** (client sends `POST` until the terminal SSE event arrives) separately, together with first-event latency, queue wait, execution time and delivery delay, and SHALL report p50, p95 and max for each.
3. THE harness SHALL write the report to `target/run-lifecycle/dod-report.json` and print a Markdown table, so the numbers are measured and not typed by hand.

### R21. Definition of done

1. A test SHALL submit **twenty lightweight runs concurrently** from twenty different users, with several judge workers running, and SHALL verify:
   - no lost jobs: 20 submissions, 20 jobs, 20 terminal runs, no job left `queued` or `claimed`;
   - no duplicate terminal results: each run has exactly one terminal event, one set of `test_results`, and a contiguous `seq` from 1;
   - no unauthorized event access: a second group of users and an unauthenticated client attempt to read every run's events, results and cancel endpoint, and every attempt returns `401` or `404` with no run data;
   - no hidden-data leak: canary markers appear nowhere in responses or stored events;
   - each client stream is ordered and de-duplicated and ends with the terminal event;
   - enqueue and completion latency are reported separately.
2. THE enqueue p95 SHALL be at most 300 ms in this test (R2.6).
3. THE twenty runs SHALL all be terminal within 60 s on the CI runner.

### R22. Required integration scenarios

THE test suite SHALL contain automated Postgres-backed tests for at least these scenarios. Each is described in `design.md` ("Integration test plan").

1. Concurrent workers claiming from a shared queue.
2. Duplicate claims (second claim, wrong-owner renewal, wrong-owner write).
3. Lost SSE connections, dropped at every position of a run's event stream.
4. Restart recovery: a worker killed mid-run, a judge fleet restarted, and a server restarted mid-stream.
5. Cancellation of a queued run, a running run, a finished run, and someone else's run.
6. Event replay with `Last-Event-ID` (header and query), with notifications enabled and disabled.
7. Retries exhausted, expiry, and executor infrastructure errors.
8. Idempotent enqueue (sequential and concurrent) and the cap and rate limit behaviour.
9. Authorization on every run endpoint.
10. Hidden-data canaries.

## Out of scope

- The sandbox runner, drivers, comparator, test storage and bench mode (judge-sandbox and language-drivers specs).
- Ghost hunting, shrink traces and Bug Autopsy payloads (ghost-engine and related specs). Their event types exist here; their payloads are defined there.
- Sessions, OAuth, CSRF middleware and the demo account (auth spec). This spec uses a `UserResolver` seam.
- The problem catalogue tables and import (a later server spec). This spec uses a `ProblemLookup` seam.
- The hourly ghost-hunt quota, the AI hint quota and the Familiar's speech bubble.
- The run history list (`GET /api/submissions`), the dashboard and the IDE panels.
