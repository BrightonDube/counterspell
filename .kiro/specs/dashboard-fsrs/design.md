# Design: dashboard-fsrs

## Design approach

This is a Requirements-First feature spec. `requirements.md` is the approved product contract; this document decides the data boundaries, query shape, API response, ordering, failure behaviour, and verification plan. It adds no crate, service, queue, event bus, or alternate repository layout.

The architecture remains the one in `docs/system-design.md`: Axum in `counterspell-server`, PostgreSQL as truth, Valkey only for cache/rate limits, SvelteKit static frontend, and the canonical six Cargo packages. The dashboard reads durable run and catalogue data. It does not execute code, own verdicts, or consume hidden tests.

No task in `tasks.md` is run by this spec-generation step.

## Plain-language design

The dashboard is a read model assembled from PostgreSQL whenever the signed-in user opens it:

1. The auth session resolves to one `UserId`.
2. The server runs one parameterised dashboard query with that `UserId` as its only user selector.
3. CTEs first isolate the user's completed submissions and accepted problem set. All later aggregates derive from those CTEs.
4. The query joins only published catalogue records for concepts, difficulties, and recommendations.
5. The server maps the result to a stable JSON response. It never returns code or hidden judge data.
6. The Svelte page displays the response with explicit loading, empty, error, and signed-out states.

FSRS is a later extension:

1. A review card belongs to exactly one user and one concept.
2. The server loads the card and review history inside a transaction, applies one `fsrs` scheduler operation, validates the returned interval, and persists the card and history together.
3. Due-review reads and grade writes always use the authenticated `UserId` and never accept an arbitrary owner id.
4. A sibling problem is selected from published catalogue rows and is not treated as solved for this user.

## Scope boundary and seams

### Provided by other specs

| Seam | Provider | Use here |
|---|---|---|
| Authenticated actor | `.kiro/specs/auth/`, through the run-lifecycle `UserResolver`/`CurrentUser` seam | Supplies the `UserId`; invalid sessions become `401 unauthenticated`. |
| Run schema and verdicts | `.kiro/specs/run-lifecycle/` | Supplies `submissions.status`, `submissions.verdict`, timestamps, counts, and ownership foreign key. This design uses lifecycle `status = completed` plus `verdict = accepted`, not the older combined-status wording in system-design §4. |
| Published problems | Grimoire/catalogue spec | Supplies problem slug, title, difficulty, publication state, track, and concept associations. This design does not create a second problem table. |
| Concept metadata | Grimoire/catalogue spec | Supplies concept slug and display name through `concepts` and `problem_concepts` or its equivalent typed catalogue seam. |
| Shared IDs and DTOs | `counterspell-common` only if the existing implementation needs cross-crate types | The server may keep dashboard response types local when no other package consumes them. |

### Owned by this spec

- `crates/server/src/dashboard/` for the core query, response mapping, handler, and API errors.
- `web/src/routes/dashboard/` and small dashboard client/types under `web/src/lib/dashboard/`.
- A dashboard index migration only where the existing run/catalogue migrations do not already provide the required indexes.
- [stretch] `crates/server/src/reviews/` for FSRS adapter, card storage, review endpoints, and scheduler properties.
- [stretch] review-card migration and review-history migration.

The server SHALL not depend on `counterspell-judge`, `bollard`, or a Docker socket for this feature.

## Canonical module map

```text
crates/server/src/
├── dashboard/
│   ├── mod.rs             # router, DashboardState, public response types
│   ├── handler.rs         # GET /api/dashboard and CurrentUser boundary
│   ├── query.rs           # one dashboard SQL query and row decoding
│   ├── recommendations.rs # pure ranking/fallback helpers, if query output needs mapping
│   └── errors.rs          # safe dashboard-specific errors, or reuse server ApiError
└── reviews/               # [stretch]
    ├── mod.rs             # review routes and state
    ├── fsrs_adapter.rs    # fsrs crate boundary and invariant checks
    ├── query.rs           # card/history/due-review queries
    └── handler.rs         # due and grade endpoints

web/src/
├── lib/dashboard/
│   ├── client.ts          # fetch and JSON error handling
│   ├── types.ts           # response types
│   └── components/        # cards, lists, empty/error states
└── routes/dashboard/
    └── +page.svelte       # authenticated dashboard screen
```

The exact TypeScript extension can follow the existing web convention. No second frontend or dashboard application is allowed.

## Core PostgreSQL data contract

The dashboard depends on these existing canonical columns rather than duplicating them:

- `users.id` and the auth/session ownership seam.
- `submissions.id`, `user_id`, `problem_slug`, `lang`, `mode`, `status`, `verdict`, `passed`, `total`, `score`, `created_at`, and `finished_at` from run-lifecycle.
- `problems.slug`, `title`, `difficulty`, and publication/status fields from the catalogue.
- `concepts.slug`, `name`, and `problem_concepts(problem_slug, concept_id)` from the catalogue.

The run-lifecycle design's lifecycle/verdict split is authoritative for implementation:

```text
completed + accepted       = accepted user solution
completed + other verdict = completed user-code failure
queued/claimed/active      = not yet an activity result
expired/internal_error     = not a user-code verdict
```

If the catalogue implementation uses a typed `ProblemRef` rather than a public `status` column, the SQL adapter SHALL restrict records to the catalogue's published set before returning them. The dashboard SHALL not create a parallel publication flag.

### Dashboard-specific indexes

The implementation SHALL inspect existing migrations before adding indexes. If they are absent, one dashboard migration may add only indexes such as:

```sql
CREATE INDEX submissions_dashboard_user_finished
    ON submissions (user_id, finished_at DESC, id DESC)
    WHERE finished_at IS NOT NULL;

CREATE INDEX submissions_dashboard_user_problem_verdict
    ON submissions (user_id, problem_slug, verdict);

CREATE INDEX submissions_dashboard_user_created
    ON submissions (user_id, created_at DESC, id DESC);
```

The migration SHALL not add a materialised dashboard table. PostgreSQL remains the truth and the response is cheap enough to derive from indexed rows at launch. Any index that already exists SHALL not be duplicated.

## Core API contract

### `GET /api/dashboard`

The endpoint is mounted under the existing `/api` router and uses the cookie-backed authenticated session. It has no path parameter and accepts no user selector.

Success status: `200`.

Illustrative response shape:

```json
{
  "solved": {
    "total": 2,
    "by_difficulty": {
      "easy": 1,
      "medium": 1,
      "hard": 0
    }
  },
  "streak": {
    "current_days": 4,
    "active_today": true,
    "last_activity_date": "2026-11-12"
  },
  "weak_concepts": [
    {
      "slug": "sliding-window",
      "name": "Sliding window",
      "failure_count": 3,
      "last_failed_at": "2026-11-11T18:20:00Z"
    }
  ],
  "recent_submissions": [
    {
      "id": "uuid",
      "problem": {
        "slug": "count-the-pairs",
        "title": "Count the Pairs",
        "difficulty": "easy"
      },
      "lang": "python",
      "mode": "submit",
      "status": "completed",
      "verdict": "accepted",
      "passed": 12,
      "total": 12,
      "score": 100,
      "created_at": "2026-11-12T18:00:00Z",
      "finished_at": "2026-11-12T18:00:01Z"
    }
  ],
  "suggested_challenges": [
    {
      "slug": "window-without-sorting",
      "title": "Window Without Sorting",
      "difficulty": "medium",
      "concepts": [{"slug": "sliding-window", "name": "Sliding window"}],
      "reason": "practice_weak_concept"
    }
  ]
}
```

The actual implementation SHALL use the existing catalogue's difficulty values. If those values are `easy`, `medium`, and `hard`, all three keys are present. If the catalogue establishes another fixed set, the response type and UI use that set consistently.

Error responses follow `.kiro/steering/api-conventions.md`:

```json
{"code":"unauthenticated","message":"Sign in to view your dashboard."}
```

- `401 unauthenticated`: no valid session.
- `503 dashboard_unavailable`: PostgreSQL read failed or the response could not be built safely.
- `500 internal_error`: only for a safe, unexpected mapping/configuration failure; the response contains no SQL detail.

The handler logs only a safe error code and tracing context. It does not log source code, session values, hidden test values, or raw user output.

### Why the dashboard has one query

The server does one ownership-checked query with CTEs and JSON aggregation. It may decode one row containing JSON arrays or several columns from the same query result, but it SHALL not issue separate database calls for solved counts, streak, weak concepts, recent submissions, and suggestions.

The query shape is:

```text
WITH
owned_completed AS (
  SELECT ...
  FROM submissions
  WHERE user_id = $1 AND status = 'completed'
),
accepted_problems AS (
  SELECT DISTINCT problem_slug FROM owned_completed WHERE verdict = 'accepted'
),
solved_counts AS (... join published problems ...),
activity_days AS (... distinct UTC date from owned_completed ...),
weak_concepts AS (... failed verdicts joined to problem_concepts ...),
recent AS (... user-owned submissions ordered by created_at, id ... LIMIT 10),
suggestion_candidates AS (... published and not in accepted_problems ...),
suggestions AS (... weak-concept score plus deterministic fallback ...)
SELECT solved_counts, streak inputs, weak_concepts, recent, suggestions;
```

The exact SQL can use helper CTEs or a SQL function, but every user-scoped CTE starts with `$1` or derives only from a previously constrained CTE. It must not first aggregate all users and filter afterward.

### Streak calculation

The query returns distinct UTC activity dates in descending order or returns the calculated count from a date-series CTE. The Rust mapper calculates the current streak with explicit date arithmetic if that makes the query easier to test:

- no dates → `0`, `active_today = false`, `last_activity_date = null`;
- latest date is today or yesterday → walk backward one day at a time while dates exist;
- latest date older than yesterday → `0`.

The date comes from `finished_at` for completed submissions. `created_at` is a fallback only if the lifecycle contract explicitly records a completed row without `finished_at`; the normal run-lifecycle invariant requires `finished_at` for terminal rows. The implementation uses UTC consistently.

### Weak-concept calculation

A failed concept event is one row per `(user, completed submission, concept)` where the submission verdict is not `accepted`. One submission can increment several concepts, but repeated rows caused by a join or duplicate concept association are de-duplicated. The query orders by:

1. `COUNT(*) DESC`;
2. `MAX(finished_at) DESC`;
3. `concept.slug ASC`.

The UI displays the count as the user's failed attempts associated with the concept, not as a universal difficulty score.

### Recommendation algorithm

The recommendation score is intentionally simple and explainable:

```text
score(problem) =
  100 * number of the user's weak concepts on the problem
  + 10  * number of the problem's concepts
  + difficulty_fallback_bonus
```

The implementation may use a pure helper with the same ordering rather than expose the numeric score. It SHALL:

1. start from published problems only;
2. exclude `accepted_problems` for this user;
3. prefer overlap with weak concepts, with the highest weak-concept failure count as the first tie-breaker;
4. use the catalogue's stable track/difficulty fallback if no weak concept overlaps;
5. resolve all remaining ties by difficulty rank and slug;
6. return no more than three items.

The user-facing reason is one of a small stable set, initially `practice_weak_concept` or `continue_track`. No LLM is used for ranking or wording.

## [stretch] FSRS data contract

FSRS work is blocked by the core freeze-safe checkpoint. It is an extension to the same server, database, and dashboard response. It does not move review data to Valkey.

### Review tables

The [stretch] migration adds the tables only after confirming the auth, concept, and account-deletion foreign keys that already exist:

```sql
CREATE TABLE concept_cards (
    user_id       uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    concept_id    text NOT NULL REFERENCES concepts(id) ON DELETE CASCADE,
    stability     double precision NOT NULL,
    difficulty    double precision NOT NULL,
    reps          integer NOT NULL DEFAULT 0,
    lapses        integer NOT NULL DEFAULT 0,
    state         text NOT NULL,
    due           timestamptz NOT NULL,
    last_review   timestamptz,
    version       bigint NOT NULL DEFAULT 0,
    created_at    timestamptz NOT NULL DEFAULT now(),
    updated_at    timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, concept_id),
    CHECK (stability >= 0),
    CHECK (difficulty >= 0),
    CHECK (reps >= 0),
    CHECK (lapses >= 0),
    CHECK (version >= 0)
);

CREATE TABLE concept_review_history (
    id                  bigserial PRIMARY KEY,
    user_id             uuid NOT NULL,
    concept_id          text NOT NULL,
    grade               text NOT NULL CHECK (grade IN ('again', 'hard', 'good', 'easy')),
    reviewed_at         timestamptz NOT NULL,
    previous_state      jsonb NOT NULL,
    next_state          jsonb NOT NULL,
    next_interval_secs  bigint NOT NULL CHECK (next_interval_secs >= 0),
    FOREIGN KEY (user_id, concept_id)
      REFERENCES concept_cards(user_id, concept_id) ON DELETE CASCADE
);

CREATE INDEX concept_cards_due ON concept_cards (user_id, due, concept_id);
CREATE INDEX concept_review_history_user_time
    ON concept_review_history (user_id, reviewed_at DESC, id DESC);
```

The exact `concept_id` type SHALL follow the catalogue migration. If concepts use slugs as their primary key, `text` is correct; if the catalogue establishes UUID ids, the implementation uses that type consistently and does not add a duplicate slug-key table.

The card stores the serialisable FSRS state needed to reconstruct the third-party `fsrs::Card`. It does not persist a Rust-specific opaque object. The history JSON records the before/after state so a future library upgrade can be audited and tested.

### `fsrs` adapter

`fsrs_adapter.rs` is the only module that knows the third-party API. It maps:

- persisted state ↔ `fsrs::Card`;
- API grade strings ↔ `fsrs` rating enum;
- scheduler state ↔ persisted `state` string;
- the application clock ↔ the scheduler's review timestamp.

The dependency is added to `counterspell-server` (or to the existing server scheduler dependency set), with its version and BSD-3-Clause notice recorded in `THIRD-PARTY-NOTICES.md`. The implementation SHALL not add `rs-fsrs`.

The adapter returns a small application-owned result, for example:

```text
ReviewCalculation {
    card: PersistedCard,
    grade: ReviewGrade,
    reviewed_at: Timestamp,
    next_interval: Duration,
    due: Timestamp,
}
```

The adapter validates before persistence:

1. `next_interval >= 0`.
2. `due >= reviewed_at`.
3. the due timestamp is representable by the database type.
4. the grade is one of the four allowed values.
5. the same input state, timestamp, and configuration produces the same result.
6. for the property fixture, `good` produces a strictly longer interval than `again` from the same valid prior state.
7. repeated good updates do not shorten the stored interval. If the FSRS library's result violates the application invariant, the transaction rolls back and returns `scheduler_invariant_failed`.

The production adapter does not silently replace the FSRS algorithm with a custom schedule. The invariant is a guard around the selected library version; changing the library version or parameters requires rerunning the property suite and reviewing the result.

### Review write order and optimistic concurrency

`POST /api/reviews/{concept}` uses a transaction:

1. authenticate and resolve `UserId`;
2. lock `concept_cards` for `(user_id, concept_id)` or create the initial card safely;
3. verify an optional client `version` matches the stored version;
4. calculate the next card through the `fsrs` adapter;
5. validate interval/due invariants;
6. update the card with `version + 1`;
7. insert history with the exact before/after state;
8. select a published sibling problem using the same user ownership filter;
9. commit and return the updated card and sibling.

A stale version returns `409 review_conflict` and writes nothing. A retry with the same version cannot create two successful state transitions. The selected sibling is a response convenience; it is not a second review record.

### [stretch] review endpoints

- `GET /api/reviews/due`: authenticated, due cards only, ordered by `due`, then concept slug. It may return a bounded page, assumed 20 items.
- `POST /api/reviews/{concept}`: authenticated grade write. Body contains `grade` and optional `version`; it does not contain `user_id`.
- [stretch] `GET /api/dashboard` adds `review_summary` only after the review migration and endpoints are ready. Before then, the core response has no fabricated review data.

Both endpoints use the stable JSON error envelope and perform ownership checks before reading or changing a card. Review output contains no source code, hidden test data, or other users' progress.

## Frontend design

The dashboard page is a calm Svelte 5 route using runes for request state. It has these core sections:

- progress summary: total solved and counts by difficulty;
- streak card: current days, active-today state, and last activity date;
- weak concepts: bounded list with counts and a text explanation;
- recent submissions: at most ten, with status text and icons;
- suggested challenges: up to three links with the reason and public problem metadata.

The page uses semantic Steel Night tokens from `docs/palette.md`, Bits UI primitives where helpful, and no component-level raw hex colours. It does not use GSAP. Reduced motion has no effect on data correctness and needs no animation path. Status is conveyed by words/icons as well as colour.

Loading and empty states are explicit:

- while fetching: skeleton or text labelled `Loading dashboard`;
- no activity: a useful empty state with a link to the catalogue, not fake counts;
- signed out: the auth flow or a safe unauthenticated message;
- failure: a retry control and the server error message safe for users;
- partial network response: no card is silently treated as zero.

[stretch] The due-review card is visibly labelled `Spaced repetition`, is separate from normal suggestions, and uses a static state under reduced motion.

## Security and privacy model

- The handler extracts the authenticated user from the cookie-backed session. It does not trust a browser-supplied user id.
- Every SQL query starts with `WHERE user_id = $1` or derives from an already ownership-filtered CTE.
- Every review mutation locks and updates `(user_id, concept_id)`, never a concept card selected by concept alone.
- Dashboard and review API responses are JSON. Errors contain stable codes and safe messages.
- No output is inserted as HTML. Problem titles, concept names, and user-visible text are escaped by normal Svelte rendering.
- No user code, hidden inputs, expected outputs, provider secrets, or session data is selected by the dashboard query.
- The server does not access a Docker daemon. No feature in this spec runs user or reference code.
- No dashboard cache is required for launch. If a later cache is introduced, its key must include the authenticated user id and invalidation must never turn another user's response into a cache hit.

## Verification design

### Core unit tests

- `streak_from_dates`: no activity, today, yesterday, gaps, multiple same-day runs, and dates older than yesterday.
- `rank_suggestions`: weak-concept overlap, solved exclusion, unpublished exclusion, fewer than three candidates, and stable slug tie-break.
- response mapping: zero counts, active status without verdict, null finish time, and bounded recent list.
- serialisation: difficulty values and timestamps use the established API representation.

### Core database tests

Database-backed tests are named with `pg_`, marked as requiring `DATABASE_URL`, and use the existing SQLx migration/test conventions. A fixture creates two users with deliberately distinct canary problem slugs, verdicts, concept failures, recent rows, and suggestions.

Required properties:

- `pg_dashboard_only_returns_authenticated_user`: user A never sees B's canaries in counts, weak concepts, recents, or suggestions.
- `pg_dashboard_counts_distinct_accepted_problems`: duplicate accepted submissions count once.
- `pg_dashboard_does_not_count_unfinished_or_infrastructure_rows`: queued, expired, and internal-error rows do not become solved/activity records.
- `pg_dashboard_returns_at_most_ten_recent_submissions`: ordering and cap are deterministic.
- `pg_dashboard_returns_at_most_three_suggestions`: solved and unpublished problems are excluded.
- `pg_dashboard_one_query_shape`: query instrumentation or a test database statement counter proves the core read is one query, not one query per card.

The tests assert canary absence, not only equal counts. This catches accidental aggregate leakage.

### [stretch] FSRS property tests

The property suite uses `proptest` and the application-owned adapter:

- **P-FSRS-1:** every accepted calculation has `next_interval.as_secs() >= 0` and `due >= reviewed_at`.
- **P-FSRS-2:** for generated valid card states in the supported range, the same timestamp and state produce a deterministic result.
- **P-FSRS-3:** the documented `again` versus `good` fixture always has `interval_good > interval_again`.
- **P-FSRS-4:** repeated valid `good` transitions do not reduce the persisted interval.
- **P-FSRS-5:** a rejected invariant or stale version leaves the card and history unchanged.
- **P-FSRS-6:** two users with the same concept slug have independent cards and due responses.

The generated state domain is bounded to values representable by the database and accepted by the selected `fsrs` crate version. Invalid arbitrary floats are rejected before reaching the scheduler rather than passed into it.

## Operational and dependency constraints

- The dashboard does not add a Cargo package or a service.
- Core server SQL uses runtime-checked `sqlx::query`/`query_as` style, consistent with run-lifecycle; it does not require a live database at compile time.
- Production Rust contains no `unwrap()` or `expect()` outside tests and preserves context at SQL, JSON, and time-conversion boundaries.
- Every new dependency is checked by `cargo deny`; the `fsrs` BSD-3-Clause notice is added in the task that introduces it.
- FSRS tasks are blocked until the core checkpoint. If the core misses the freeze-safe checkpoint, the FSRS slice is cut without weakening the dashboard.

## Open questions

Each question has the choice assumed by this design. Brighton can change a choice before implementation; the affected requirement and task must then be updated.

1. **What counts for a streak?** The system design stores `last_active`, while the requirements call it a streak without defining whether a failed attempt counts. **Assumed choice:** any completed Run or Submit creates a practice day, because the streak measures learning activity; accepted-only progress remains the separate solved metric.
2. **Which timezone defines a day?** Users have a country but no required timezone field in the current model. **Assumed choice:** use UTC for activity dates and streak boundaries until a timezone preference is added. This keeps results deterministic across devices.
3. **What is the exact catalogue difficulty set?** The system design examples use easy/medium/hard, while the catalogue spec owns the final enum. **Assumed choice:** use the catalogue's authoritative set; the initial response and UI support easy, medium, and hard with zero values included.
4. **Should dashboard progress be cached in Valkey?** Valkey may cache data, but user-scoped cache mistakes create privacy risk and a fresh dashboard is a launch requirement. **Assumed choice:** do not cache the dashboard at launch; PostgreSQL is queried directly. A later per-user cache requires an explicit invalidation and privacy design.
5. **Should FSRS due data be part of the core dashboard response before FSRS ships?** The full system-design dashboard view mentions a next review, but product requirements mark FSRS as stretch. **Assumed choice:** core responses contain only the must-have fields. The review summary is added as an additive field only after [stretch] migrations, endpoints, and tests are complete.
6. **Which FSRS crate API/version should be used?** The required crate is fixed, but the exact API surface may change before implementation. **Assumed choice:** pin the current workspace-compatible `fsrs` release during the dependency task, wrap it behind `fsrs_adapter.rs`, and persist application-owned primitives rather than third-party structs. `rs-fsrs` is prohibited.
7. **How should an initial concept card be created?** A review may arrive before a card exists. **Assumed choice:** create the default FSRS card inside the same transaction as the first grade, then insert the first history row; never create a card from a GET request.
8. **How should a sibling be chosen when every problem for a concept is solved?** A due review still needs a useful next action. **Assumed choice:** prefer an unsolved problem sharing the concept; if none exists, use the normal published challenge fallback and clearly label it as a general challenge rather than a sibling.
9. **What should happen when FSRS violates the product interval invariant?** Silently clamping would make the persisted state differ from the library calculation. **Assumed choice:** reject and roll back the review with `scheduler_invariant_failed`, log a safe diagnostic, and keep the prior card unchanged.
10. **Where should shared response types live?** A dashboard is currently consumed only by the server and web app. **Assumed choice:** keep server DTOs local initially; move only genuinely shared identifiers or event-independent types to `counterspell-common` if another Rust package needs them.
