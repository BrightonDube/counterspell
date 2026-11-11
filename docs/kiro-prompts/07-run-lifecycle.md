# D.7 Spec 5: `run-lifecycle` (Feature, Design-First High-Level)

Paste the block below into the Kiro IDE (new spec / Plan mode as named in the title). Model and mode: see docs/kiro-playbook.md A.2.

*Rewritten 12 Nov from critique §5.6, with Brighton's decisions. Renamed from `runs-streams-sse`.*
```
Create a Design-First (high-level) feature spec named run-lifecycle. Context: #[[file:docs/system-design.md]] §2, §4 (jobs, run_events), §5 Run lifecycle, §9, and #[[file:docs/requirements.md]] IDE-3 and VAL-4.
Use PostgreSQL as the authoritative store for submissions, the job queue and run events. Valkey is used only for rate limits and caches; do not use Valkey Streams or pub/sub as a queue or event bus.
Required behaviour:
1. Creating a submission and its claimable job is atomic (one transaction); jobs.submission_id is unique and POST /api/runs honours an Idempotency-Key.
2. A worker claims a job exclusively with SELECT … FOR UPDATE SKIP LOCKED and a renewable lease.
3. A worker crash never leaves a submission permanently running: a reaper requeues expired leases with attempts + 1, and after max_attempts the job is dead and the submission an internal error.
4. Retries are idempotent: the terminal write (final verdict, test results, done event, job succeeded) is one transaction guarded by the current lease owner, so duplicate terminal results are impossible.
5. Every public event is persisted in run_events with a monotonically increasing per-run seq. A retried attempt appends a `restarted` event first.
6. The SSE endpoint authorizes ownership before exposing any event.
7. The SSE id is seq; on reconnect the server replays events with seq > Last-Event-ID from Postgres, then waits on LISTEN/NOTIFY (a wake-up hint only) with a short poll fallback.
8. The client deduplicates by seq.
9. Hidden test inputs and expected answers are never in public event payloads.
10. A finished run has exactly one authoritative final result and terminal state, fetchable after page reload via GET /api/runs/{id}.
Define the full run state machine: queued, claimed, compiling, running, ghost_hunting, completed (with verdict), failed, expired, internal_error.
Include correctness properties and integration tests for concurrent workers, duplicate claims, lost SSE connections, restart recovery, cancellation and event replay. Per-user rate limits and the queued-jobs cap come from §7.1.
Definition of done: twenty concurrently submitted lightweight jobs are all processed with no lost jobs, no duplicate terminal results and no unauthorized event access. Report measured enqueue and completion latencies separately.
```
