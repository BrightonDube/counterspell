# dashboard-fsrs blockers (run of 15 Nov 2026, attempt 1)

## T03: one-query dashboard read model (BLOCKED, left unchecked)

- **Reason:** the query needs catalogue data that does not exist. It needs published problems with
  their titles and difficulty, concepts and the problem-to-concept mapping, for solved counts by
  difficulty, weak concepts, recent-submission titles and suggestions. The database has only
  `submissions`, `jobs`, `run_events`, `test_results`, auth tables, `run_timings` and
  `problem_baselines`. There is no `problems`, `concepts` or `problem_concepts` table.
- **Why I did not build it:** the catalogue spec (`.kiro/specs/problem-catalogue/`) is spec-only
  under Brighton's 14 Nov 2026 decision, "do not implement them". Creating catalogue tables here
  would implement that spec. The only catalogue seam, `ProblemLookup::published(slug)`, returns a
  slug and nothing else, so it cannot supply difficulty, titles or concepts.
- **Unblock:** Brighton approves `problem-catalogue`. Its migrations then create `problems`,
  `concepts` and `problem_concepts`, and `PgProblemLookup` (or an equivalent typed catalogue read)
  replaces `EmptyCatalogue` in production. After that, T03 can run against the real schema.
- **Assumed choice when unblocked:** difficulty values are `easy`, `medium` and `hard` (CAT-S7 of
  the catalogue spec). That matches the enum the dashboard response types already use.

### Tasks blocked by T03 (not started)

| Task | Blocked because |
|---|---|
| T05 wire `GET /api/dashboard` | needs the T03 read model |
| T06 `pg_` privacy and aggregate tests | needs T03 and T05. SUPERVISED, so the review queue entry waits for it |
| T07 HTTP contract and failure tests | needs T05 |
| T08 core freeze-safe checkpoint | needs T06 and T07. SUPERVISED. Cannot pass until T03 to T07 exist, so the [stretch] slice (T11 onwards) stays closed |
| T09 core dashboard screen | needs T05 and T07 |
| T10 core smoke path | needs T06 to T09 |

No [stretch] task (T11 to T18) was started, as the instructions require.

## Notes that do not block a task

- **Error code for signed-out requests.** The shared `CurrentUser` extractor sends
  `401 unauthorized`. DASH-1.2 and T01 require `unauthenticated`. The dashboard extractor
  (`DashboardUser`) reuses `CurrentUser` and only changes the code. Other API routes keep
  `unauthorized`. Brighton may want the code unified across the API. That is an api-conventions
  decision, not a dashboard one.
- **Timestamps and dates in the response are strings** (RFC 3339 UTC, and `YYYY-MM-DD`). The
  server does not enable the `time` crate's serde feature, and I did not add a feature flag the
  task did not name.
- **Suggestion reason `continue_track` is a fallback label.** Track data is not in the candidate
  set yet, so every suggestion with no weak-concept overlap is labelled `continue_track`. When the
  catalogue lands, the track order should decide the fallback, and the label should be checked
  against that.
- **Log content on `DashboardError::unavailable`.** The function logs the full `Display` of the
  source error. Postgres error text names tables and constraints, not row data. T03 should still
  pass only the sqlx error, never a row or request body, into this function.
- **Index choice (T02).** The design's example index on `(user_id, problem_slug, verdict)` was
  replaced by a partial index `(user_id, problem_slug) WHERE verdict = 'accepted'`. It covers the
  distinct solved set with fewer rows, and the verdict column is not needed in the index because
  the predicate fixes it.

## Attempt 2 (15 Nov 2026, later run): no change to the T03 block

- Checked again: `migrations/` still has no `problems`, `concepts` or `problem_concepts` table, and
  the only catalogue commits are the spec commits (`4e2fd2b`, `69e42b0`, `c94504f`). Brighton's
  14 Nov 2026 rule keeps `problem-catalogue` spec-only, so T03 stays unchecked.
- Every unchecked Must task depends on T03 (T05 through T10), and the stretch gate opens only after
  T08. So this run has no task it can start without faking catalogue data. No code was written.
- Unblock: Brighton approves `problem-catalogue` (or confirms that a minimal `problems` and
  `problem_concepts` read contract may ship inside this spec). Then T03 can start on the real schema.
- Tasks T01, T02 and T04 remain ticked from attempt 1 and are unchanged.
