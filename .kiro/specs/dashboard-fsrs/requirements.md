# Requirements: dashboard-fsrs

## Purpose

Add the signed-in learner dashboard to Counterspell. The must-have dashboard shows progress that can be derived from the learner's own completed runs: solved counts, the practice streak, weak concepts, recent submissions, and three deterministic suggested challenges. Spaced repetition is a separate stretch slice. It schedules concept reviews with the `fsrs` crate after the core dashboard is freeze-safe.

This is a **Requirements-First** feature spec. The files in this directory define the feature; this spec-generation run executes no task and changes no implementation file.

## Sources and fixed decisions

- `docs/system-design.md` §1, §4, and §9.
- `docs/product-brief.md`, core loop step 6.
- `docs/requirements.md` E2 (`PRB-2`) and E7 (`DSH-1`, `DSH-2`, `DSH-3`).
- `.kiro/steering/api-conventions.md`, `frontend.md`, `product.md`, `security.md`, `structure.md`, `tech.md`, and `rust-teaching.md`.
- `.kiro/specs/repo-skeleton/` for the canonical workspace and package names.
- `.kiro/specs/auth/` for the authenticated-user seam and account data.
- `.kiro/specs/run-lifecycle/` for the lifecycle/status/verdict split and the `UserResolver` seam.

The feature SHALL keep the six-package workspace:

- `crates/server/` → `counterspell-server`: dashboard API, dashboard query, and review API.
- `crates/judge/` → `counterspell-judge`: unchanged by this feature.
- `crates/engine/` → `counterspell-engine`: unchanged by this feature.
- `crates/grimoire/` → `counterspell-grimoire`: provides published problem and concept metadata through the catalogue seam.
- `crates/mcp/` → `counterspell-mcp`: unchanged by this feature.
- `crates/common/` → `counterspell-common`: only shared dashboard/review DTOs if a later implementation proves they cross the server boundary.

The spaced-repetition implementation SHALL use the `fsrs` crate, licensed BSD-3-Clause. It SHALL NOT use `rs-fsrs`.

## Scope and priority

### Must: dashboard core

- `GET /api/dashboard` for the authenticated user.
- Solved total and solved counts by difficulty.
- Current practice streak.
- Weak concepts based on that user's failed judged submissions.
- The ten most recent submissions owned by that user.
- Up to three suggested published challenges, with deterministic ordering and a reason.
- A SvelteKit dashboard screen that displays these values accessibly and does not fabricate values when the API is unavailable.
- Ownership, authorization, stable JSON errors, and database tests for cross-user isolation.

### [stretch]: spaced repetition

- Persist FSRS concept cards and review history.
- Grade a concept review with `again`, `hard`, `good`, or `easy`.
- Calculate a non-negative next interval and due date through `fsrs`.
- Show due concept reviews and choose a fresh sibling problem.
- Add the due-review card to the dashboard without weakening the must-have dashboard.

The [stretch] work SHALL NOT start until the dashboard core passes its freeze-safe checkpoint. The core checkpoint includes the API, query, ownership tests, frontend loading/error/empty states, and the existing CI-equivalent checks. FSRS is not a prerequisite for the core dashboard.

## Glossary

- **Accepted problem:** a published problem for which the user has at least one completed submission with `verdict = accepted`.
- **Activity day:** a UTC calendar date on which the user has at least one completed Run or Submit. Infrastructure failures, queue expiry, and runs that have not finished do not create an activity day.
- **Current streak:** the number of consecutive activity days ending today, or ending yesterday when the user has not yet practised today. A user with no activity has a zero streak.
- **Weak concept:** a published problem concept associated with one or more of the user's completed non-accepted verdicts. Infrastructure failures are not concept failures.
- **Suggested challenge:** an eligible published problem the user has not solved, ranked from the user's weak concepts and deterministic catalogue fallbacks.
- **Review card:** [stretch] one user's FSRS state for one concept.
- **Next interval:** [stretch] the non-negative duration from the review time to the newly calculated due time.

## Requirements

### DASH-1: authenticated dashboard

**User story:** As a learner, I want to see my progress in one dashboard request, so that I know what to practise next.

1. WHEN an authenticated user requests `GET /api/dashboard`, THE SYSTEM SHALL return JSON containing the user's solved counts, current streak, weak concepts, ten most recent submissions, and up to three suggested challenges.
2. IF the request has no valid authenticated session, THEN THE SYSTEM SHALL return `401` with the stable JSON error code `unauthenticated` and SHALL return no dashboard data.
3. THE SYSTEM SHALL not accept a user id, email, handle, or account selector in the dashboard URL or request body. The authenticated session is the only source of the dashboard owner.
4. THE SYSTEM SHALL read dashboard facts from PostgreSQL. Valkey SHALL not be the source of dashboard truth.
5. THE SYSTEM SHALL return a stable JSON success shape even when the user has no submissions: counts and streak are zero, arrays are empty, and no placeholder submission or challenge is presented as real data.

### DASH-2: solved counts

**User story:** As a learner, I want to see what I have solved, so that I can measure progress by difficulty.

1. THE SYSTEM SHALL include `solved.total` and `solved.by_difficulty` in the response.
2. `solved.by_difficulty` SHALL include the catalogue's supported difficulty values and a count for each value, including zero counts.
3. A problem SHALL count at most once per user, even if the user has multiple accepted submissions for it.
4. A problem SHALL count as solved only when the user's own completed submission has `verdict = accepted`.
5. THE SYSTEM SHALL not count another user's accepted submission, a hidden test result that the user does not own, a queued run, an expired run, or an infrastructure error.

### DASH-3: practice streak

**User story:** As a learner, I want a streak that reflects practice, so that I have a reason to return.

1. THE SYSTEM SHALL calculate the current streak from distinct UTC activity dates belonging to the authenticated user.
2. A completed Run or Submit creates an activity day regardless of whether its user-code verdict was accepted or failed. Queue expiry, infrastructure failure, and an unfinished run do not create an activity day.
3. The streak SHALL be zero for a user with no completed activity.
4. The streak SHALL count consecutive dates ending today, or ending yesterday if today has no activity, and SHALL stop at the first missing date.
5. The API SHALL include the streak length, whether the user was active today, and the last activity date or `null`.

### DASH-4: weak concepts

**User story:** As a learner, I want to know which concepts cause me trouble, so that I can target practice.

1. THE SYSTEM SHALL associate a failed completed verdict with every published concept attached to that submission's problem.
2. The weak-concept list SHALL be scoped to the authenticated user and SHALL exclude infrastructure failures that have no user-code verdict.
3. Each weak concept SHALL include a stable concept slug, display name, failure count, and the date of the user's most recent failure.
4. The API SHALL order weak concepts by failure count descending, most recent failure descending, then concept slug ascending.
5. The API SHALL cap the list at a documented response limit, assumed to be ten, and SHALL not reveal another user's failure counts or review state.
6. A concept with no user failure SHALL not appear as weak merely because another user struggled with it.

### DASH-5: recent submissions

**User story:** As a learner, I want to see recent attempts, so that I can return to unfinished learning work.

1. THE SYSTEM SHALL return at most ten submissions for the authenticated user, newest first by creation timestamp and then id.
2. Each recent item SHALL include only display-safe fields: run id, problem slug and title, difficulty, language, mode, lifecycle status, verdict when present, passed count, total count, score, created time, and finished time.
3. The response SHALL not include source code, hidden test input, hidden expected output, private judge diagnostics, session data, or another user's submission.
4. A recent item whose run is still active SHALL display its lifecycle status and SHALL not invent a verdict.
5. The query SHALL use the run-lifecycle contract: lifecycle is in `submissions.status`, while the user-code verdict is in `submissions.verdict`.

### DASH-6: suggested challenges

**User story:** As a learner, I want three useful next problems, so that I do not waste time choosing.

1. THE SYSTEM SHALL return at most three published problems that the authenticated user has not solved.
2. The recommendation rank SHALL prefer problems sharing concepts with the user's weak concepts, then use deterministic catalogue fallbacks when there are too few matches.
3. The response SHALL include a stable problem slug, title, difficulty, concept summaries, and a short reason such as `practice_weak_concept` or `continue_track`.
4. The recommendation query SHALL exclude unpublished problems, malformed catalogue records, and problems already solved by the authenticated user.
5. Ties SHALL be resolved deterministically by difficulty rank, then problem slug. The same database state and current date SHALL produce the same ordered suggestions.
6. Suggested challenge data SHALL be public catalogue data plus the authenticated user's own progress signal. It SHALL not expose aggregate statistics about other users.

### DASH-7: one dashboard read and failure behaviour

1. THE SYSTEM SHALL fulfil the core dashboard response with one authenticated server request and one PostgreSQL read operation, implemented as one query or SQL function with CTEs/aggregates.
2. The server SHALL not make one query per card in the dashboard.
3. If PostgreSQL is unavailable or the dashboard query fails, THE SYSTEM SHALL return a stable `503 dashboard_unavailable` JSON error and SHALL not return a partial response labelled complete.
4. The server SHALL log a safe diagnostic code and context without source code, session values, or private submission output.
5. The dashboard endpoint SHALL not use SSE; dashboard refresh is a normal JSON fetch. Run completion remains owned by the run-lifecycle event flow.

### DASH-8: ownership and privacy properties

1. For any two users A and B with different submissions, concepts, and review cards, a dashboard request authenticated as A SHALL contain no row or aggregate derived from B.
2. Every user-scoped SQL path SHALL constrain by the authenticated `UserId` before aggregating, and joins SHALL not remove that constraint.
3. A guessed or substituted UUID SHALL not let a user read another user's dashboard or review data.
4. The implementation SHALL prove the property with a database integration test containing distinct canary values for two users.
5. No dashboard or review response SHALL contain raw passwords, provider ids, session data, code, hidden tests, or secrets.

### [stretch] FSRS-1: card and review storage

1. [stretch] THE SYSTEM SHALL store one FSRS card per `(user_id, concept_id)` with the fields required by the `fsrs` crate and the system design: stability, difficulty, repetitions, lapses, state, due, and last review.
2. [stretch] Review history SHALL record the owning user, concept, grade, review time, previous card state, next card state, and calculated interval for auditability.
3. [stretch] The database SHALL enforce one card per user/concept and foreign keys SHALL cascade only the owning user's records when account deletion is implemented.
4. [stretch] The schema SHALL not store a second scheduler state in Valkey or in process memory as the source of truth.

### [stretch] FSRS-2: review grading and interval properties

1. [stretch] The server SHALL accept only the four FSRS grades `again`, `hard`, `good`, and `easy`.
2. [stretch] The review calculation SHALL use the `fsrs` crate and SHALL not use `rs-fsrs` or a hand-rolled replacement while the crate is available and licensed.
3. [stretch] For every accepted review, `next_interval` SHALL be non-negative and `due` SHALL not be before the review timestamp.
4. [stretch] For the same valid pre-review card and review timestamp, a `good` grade SHALL produce a strictly longer next interval than an `again` grade in the scheduler adapter's property fixture.
5. [stretch] Repeated valid `good` reviews on a stable card SHALL not shorten the persisted interval. If the third-party result violates the adapter invariant, the server SHALL reject the update without changing the card and return a safe scheduler error.
6. [stretch] The calculation SHALL be deterministic for the same card, grade, timestamp, and scheduler configuration.

### [stretch] FSRS-3: due reviews and sibling challenge

1. [stretch] `GET /api/reviews/due` SHALL return only the authenticated user's due cards, ordered by due time and concept slug.
2. [stretch] A due review SHALL include a fresh published sibling problem that shares the concept and is not solved by the user when one exists.
3. [stretch] `POST /api/reviews/{concept}` SHALL require authentication, verify the concept exists, apply the grade atomically, and return the updated card plus the selected sibling problem.
4. [stretch] The endpoint SHALL reject duplicate or stale writes according to the review version/last-review guard chosen in design, without corrupting the card.
5. [stretch] Review endpoints SHALL use JSON success and stable JSON errors and SHALL have ownership checks before returning or changing any card.

### [stretch] FSRS-4: dashboard integration

1. [stretch] Once FSRS is enabled, `GET /api/dashboard` SHALL include a due-review summary without changing the meaning of the core fields.
2. [stretch] If no cards are due, the response SHALL use an empty or null review value and SHALL not fabricate a next review.
3. [stretch] The dashboard SHALL label the review item as spaced repetition and distinguish it from a normal suggested challenge.

### DASH-9: accessible frontend

1. WHEN an authenticated learner opens the dashboard, THE SYSTEM SHALL render the core cards using Svelte 5 runes, Bits UI where a primitive is needed, Tailwind 4 semantic Steel Night tokens, and keyboard-accessible controls.
2. The UI SHALL show text and accessible labels in addition to status colour; bright green is reserved for success and red for failure.
3. The UI SHALL provide loading, empty, unauthenticated, and error states. It SHALL not display sample fixtures or fabricated run results as real progress.
4. The core dashboard SHALL remain usable with reduced motion and SHALL not add ambient IDE animation.
5. [stretch] The due-review card SHALL also have a static/reduced-motion presentation and an `aria-live`-safe update if a review is submitted.

## Out of scope

- Authentication, OAuth, sessions, account linking, and demo-account policy. The auth spec provides the authenticated-user seam.
- Run execution, verdict production, ghost hunting, bench fitting, and SSE. The run-lifecycle and later judge specs own those facts.
- Problem authoring or importing. The Grimoire/catalogue specs own publication, difficulty, concepts, and sibling selection metadata.
- Instructor/cohort dashboards, readiness scores, payments, admin analytics, and cross-user leaderboards.
- Night Shift, FSRS reviews before the core freeze-safe checkpoint, Java/C++ support, and the break room.
