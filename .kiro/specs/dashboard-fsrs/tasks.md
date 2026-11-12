# Tasks: dashboard-fsrs

All tasks below are implementation tasks for `requirements.md` and `design.md`. No task is executed by this spec-generation run. Each task is sized for 30 minutes or less. `[P]` means it may run in parallel with other tasks in its wave after its dependencies complete. `[S]` means sequential. `[SUPERVISED]` marks an ownership, migration, or scheduler change that Brighton should review before dependent work proceeds.

## Rules for every task

- Preserve the canonical six-package workspace and existing crate boundaries. Do not add a package, service, queue, event bus, or alternate frontend.
- The core dashboard is a Must. The FSRS slice is `[stretch]` and SHALL NOT begin before T08's freeze-safe core checkpoint passes.
- Follow `.kiro/steering/api-conventions.md`, `frontend.md`, `product.md`, `security.md`, `structure.md`, `tech.md`, and `rust-teaching.md`.
- Every user-scoped query and every dashboard/review endpoint must derive ownership from the authenticated session. Never accept a user id from the browser.
- PostgreSQL is truth. Valkey remains cache/rate-limit storage only and is not needed for the initial dashboard.
- Use runtime-checked SQLx queries, not compile-time database macros, unless the existing server convention changes explicitly.
- Do not read, write, or create `.env`, credential, key, or secret files.
- Production Rust must not use `unwrap()` or `expect()` outside tests and must preserve error context at SQL, JSON, and time boundaries.
- Every Rust task ends with a 3–5 line plain-English Rust lesson in the task reply, covering the ownership, borrowing, `Result`/`?`, async, or trait idea that matters.
- Database-backed tests use the repository's `pg_` naming and ignored-test conventions. Tests must use distinct users and canary values to prove privacy, not only equal counts.
- The FSRS dependency is exactly `fsrs` with its BSD-3-Clause notice. Do not add or mention `rs-fsrs` as an implementation option.

## Wave 0: core contracts and query foundations

- [ ] **T01 [S] (25 min) Confirm feature seams and dashboard response types**
  - Files: `crates/server/src/dashboard/{mod.rs,types.rs,errors.rs}`, and the existing server router/seam files only where route registration requires it.
  - Add the dashboard module boundary, authenticated handler input, response structs, stable empty response shape, and error mapping for `unauthenticated`, `dashboard_unavailable`, and safe internal failures.
  - Reuse the existing `CurrentUser`/`UserResolver` seam and the run-lifecycle lifecycle/verdict split. Do not introduce a second auth extractor or a user-id request field.
  - Verify: server type-checks; unauthenticated requests map to JSON `401`; response serialization includes zero/empty values without fabricated rows.
  - Requirements: DASH-1, DASH-5, DASH-7, DASH-8.
  - Depends on: auth and run-lifecycle seams being available; no database migration required.
  - Rust lesson: explain how a typed response struct makes the JSON contract explicit and how `Result` carries safe handler failures.

- [ ] **T02 [P] (25 min) Add only the required dashboard indexes**
  - Files: `migrations/<timestamp>_dashboard_indexes.sql` if an inspection shows the run-lifecycle/catalogue indexes are insufficient.
  - Add user/finished-time, user/problem/verdict, and user/created-time indexes only when absent. Do not create a materialised dashboard table or duplicate existing indexes.
  - Verify: migration applies twice through SQLx; `pg_indexes` shows the expected index coverage; the migration contains no secrets and no user data.
  - Preserve: PostgreSQL remains the dashboard source of truth; Valkey is not involved.
  - Requirements: DASH-7.
  - Depends on: run-lifecycle and catalogue migrations being known.
  - Rust lesson: none for SQL-only work; in the task reply explain how an index narrows reads without becoming a second source of truth.

- [ ] **T03 [S] (30 min) Implement the one-query dashboard read model**
  - Files: `crates/server/src/dashboard/query.rs`, plus a small test fixture module if needed.
  - Implement one parameterised PostgreSQL query/function with ownership-filtered CTEs for accepted problems, solved counts, UTC activity dates/streak inputs, weak concepts, ten recent submissions, and up to three suggestions.
  - Start each user-scoped path with the authenticated `UserId` or derive it from an already constrained CTE. Use the catalogue's published problem/concept seam and the run-lifecycle `status = completed` plus `verdict` fields.
  - Keep hidden test data, code, private judge diagnostics, and another user's aggregates out of the selected columns.
  - Verify: runtime SQL decoding succeeds with fixtures; no per-card query loop exists; query returns stable empty results for a new user.
  - Requirements: DASH-2 through DASH-7 and DASH-8.
  - Depends on: T01, T02, run-lifecycle tables, and catalogue schema.
  - Rust lesson: explain borrowing a `UserId` into a SQL query, why parameterisation matters, and how `?` preserves database errors.

- [ ] **T04 [P] (25 min) Add pure streak and recommendation helpers**
  - Files: `crates/server/src/dashboard/recommendations.rs` and/or `streak.rs`.
  - Implement UTC streak calculation for no activity, today, yesterday, gaps, duplicate same-day activity, and old activity. Implement deterministic suggestion ranking, solved/unpublished exclusion, weak-concept reasons, and slug tie-breaks.
  - Keep helpers independent of Axum and PostgreSQL so property/unit tests can cover them directly.
  - Verify: unit tests cover the edge cases and repeated calls with the same input return the same ordering.
  - Requirements: DASH-3, DASH-6.
  - Depends on: T01.
  - Rust lesson: explain iterators over borrowed date data, owned return values, and why a pure function is easier to test than a database handler.

## Wave 1: core API and privacy proof

- [ ] **T05 [S] (25 min) Wire `GET /api/dashboard`**
  - Files: `crates/server/src/dashboard/handler.rs`, `dashboard/mod.rs`, and the existing API router registration.
  - Add the authenticated GET route with no user selector. Execute the one-query read model, map it to the response, and return stable JSON errors without SQL details.
  - Ensure a failed database read returns `503 dashboard_unavailable` rather than a partial dashboard. Do not add a Valkey cache in this task.
  - Verify: route registration has API precedence; signed-out requests return `401`; a signed-in fixture receives `200` with all core sections.
  - Requirements: DASH-1, DASH-7, DASH-8.
  - Depends on: T01, T03, T04 and existing auth/router seams.
  - Rust lesson: explain Axum extractors, async `Result` handling, and why the handler must not trust browser-supplied ownership data.

- [ ] **T06 [P] (30 min) Add dashboard database isolation and aggregate tests** [SUPERVISED]
  - Files: `crates/server/tests/dashboard.rs` or the existing server integration-test location.
  - Add ignored `pg_` tests with two users and distinct canary slugs/concepts. Prove user A cannot see user B's solved counts, weak concepts, recent submissions, or suggested challenges.
  - Cover duplicate accepted submissions counting once, unfinished/expired/internal-error rows not counting as solved or activity, ten recent rows, and three suggestions excluding solved/unpublished problems.
  - Verify: tests fail if the ownership predicate is removed or applied after an aggregate; tests assert canary absence, not only count equality.
  - Requirements: DASH-2 through DASH-8.
  - Depends on: T03 and T05.
  - Rust lesson: explain test fixture ownership, `Option` for nullable database values, and why canary assertions catch aggregate leakage.

- [ ] **T07 [P] (25 min) Add core API contract and failure tests**
  - Files: `crates/server/tests/dashboard_api.rs` or the existing HTTP test location.
  - Test JSON field names, zero/empty response shape, stable `401` and `503` errors, active submission with no verdict, and safe output that contains no code or hidden-test fields.
  - Verify: tests use the repository's authenticated test resolver and do not call external OAuth, Resend, Valkey, or Docker services.
  - Requirements: DASH-1, DASH-5, DASH-7, DASH-8.
  - Depends on: T05 and the existing server testkit.
  - Rust lesson: explain borrowing a test response body, `serde_json` value ownership, and how stable error codes decouple clients from SQL details.

- [ ] **T08 [S] (30 min) Core freeze-safe checkpoint** [SUPERVISED]
  - Files: only implementation fixes required by this checkpoint; no planned feature expansion.
  - Run the dashboard core unit, database, and HTTP tests, plus `cargo fmt --all -- --check`, Clippy with warnings denied, workspace tests, frontend check/build, `cargo deny check licenses`, and `docker compose config` as available in the current repository.
  - Confirm the core API is authenticated, one-query, privacy-tested, deterministic, and free of fabricated data. Confirm the server has no Docker dependency/socket and Valkey remains unused for truth.
  - Verify: all must-have DASH requirements pass; record any failure before beginning [stretch] work.
  - Requirements: all Must requirements, especially DASH-1 through DASH-9.
  - Depends on: T02 through T07 and the existing auth/run/catalogue foundations.
  - Preserve: if this checkpoint fails, stop and fix the core or cut FSRS; do not start T09 or later.
  - Rust lesson: explain how CI checks provide independent evidence and why a feature gate protects the freeze-safe product scope.

## Wave 2: core frontend, after API tests are green

- [ ] **T09 [S] (30 min) Build the core dashboard screen**
  - Files: `web/src/routes/dashboard/+page.svelte`, `web/src/lib/dashboard/{client.ts,types.ts}`, and small components under `web/src/lib/dashboard/components/`.
  - Fetch `GET /api/dashboard` with the existing cookie session, display solved counts, streak, weak concepts, ten recent submissions, and suggested challenges.
  - Implement loading, no-activity, unauthenticated, retry/error, and active-run states. Do not use fixture values as production data and do not expose code or hidden outputs.
  - Use Svelte 5 runes, Bits UI where appropriate, Tailwind 4 semantic Steel Night tokens, keyboard-visible focus, text/icons in addition to colour, and no GSAP/ambient motion.
  - Verify: `npm run check` and `npm run build`; manual or component checks prove empty/error/loading states and no raw component hex colours.
  - Requirements: DASH-1 through DASH-6 and DASH-9.
  - Depends on: T05 and T07.
  - Rust lesson: none; in the task reply note that this task has no Rust change and explain how the browser treats the API response as untrusted data.

- [ ] **T10 [S] (25 min) Run the core dashboard smoke path**
  - Files: only fixes required by the smoke path.
  - Sign in with the existing test/demo path, create or use owned submissions, request the dashboard through the Axum server, and verify the page shows real counts and suggestions. Repeat with a second user and confirm no canaries cross the boundary.
  - Verify: route works through the canonical static server path; the page does not require an npm dev server; a database/API failure produces the designed retry state.
  - Requirements: DASH-1, DASH-7, DASH-8, DASH-9.
  - Depends on: T06, T07, T08, and T09.
  - Rust lesson: explain any Rust fixes made; if no Rust file changes are needed, say so rather than adding an artificial lesson.

## Wave 3: [stretch] FSRS foundation, only after T08

- [ ] **T11 [stretch] [S] (25 min) Add the `fsrs` dependency and notices**
  - Files: `crates/server/Cargo.toml`, `Cargo.lock`, `THIRD-PARTY-NOTICES.md`, and existing licence configuration only if the dependency's declared BSD-3-Clause metadata requires it.
  - Add the workspace-compatible `fsrs` crate with the smallest required feature set. Do not add `rs-fsrs`. Record the BSD-3-Clause notice and inspect the resolved dependency tree.
  - Verify: `cargo check -p counterspell-server`; `cargo deny check licenses`; dependency tree contains `fsrs` and no `rs-fsrs`.
  - Requirements: [stretch] FSRS-2.2.
  - Depends on: T08 only.
  - Rust lesson: explain Cargo dependencies/features and why a third-party scheduler is kept behind a small adapter trait/module.

- [ ] **T12 [stretch] [S] (30 min) Add FSRS card and history migrations** [SUPERVISED]
  - Files: `migrations/<timestamp>_concept_cards.sql`, `migrations/<timestamp>_concept_review_history.sql`, and database tests.
  - Add one card per `(user_id, concept_id)`, serialised FSRS fields, due/last-review timestamps, optimistic `version`, review history, non-negative database checks, and ownership foreign keys. Match the final catalogue concept key type; do not duplicate the concepts table.
  - Verify: migrations apply twice; duplicate cards fail; negative intervals/stability/reps/lapses fail; user deletion cascades only that user's cards/history; no card exists without a valid concept/user.
  - Requirements: [stretch] FSRS-1.
  - Depends on: T11 and the auth/catalogue migration contracts.
  - Rust lesson: explain how database constraints complement Rust validation and how a composite key represents ownership plus concept identity.

- [ ] **T13 [stretch] [P] (30 min) Implement the FSRS application adapter**
  - Files: `crates/server/src/reviews/{mod.rs,fsrs_adapter.rs}`, with unit/property tests.
  - Map persisted primitives to/from the selected `fsrs` API. Accept only `again`, `hard`, `good`, and `easy`. Return an application-owned calculation with non-negative interval and due timestamp.
  - Keep the third-party API isolated to this module. Reject invalid scheduler output rather than silently replacing or clamping the algorithm. Use an injected clock/configuration for deterministic tests.
  - Verify: supported card states round-trip; malformed/unsupported states return safe errors; no production `unwrap()`/`expect()`; no `rs-fsrs` import.
  - Requirements: [stretch] FSRS-2.
  - Depends on: T11.
  - Rust lesson: explain traits as a boundary around the scheduler, borrowing versus owned card state, and `Result` for invariant failures.

- [ ] **T14 [stretch] [P] (30 min) Add FSRS property tests**
  - Files: `crates/server/src/reviews/fsrs_adapter.rs` tests or `crates/server/tests/fsrs_properties.rs`.
  - Add `proptest` properties for non-negative intervals, `due >= reviewed_at`, deterministic repeated calculations, `good` longer than `again` for the documented fixture, repeated good reviews not shortening the persisted interval, and invalid results being rejected.
  - Bound generated states to database-representable values and keep the test oracle independent of the implementation's internal helper.
  - Verify: property tests pass repeatedly with a fixed failure seed recorded by the test framework; no test depends on the wall clock or external API.
  - Requirements: [stretch] FSRS-2.3 through FSRS-2.6.
  - Depends on: T13.
  - Rust lesson: explain property-based testing, borrowing generated values for a calculation, and why deterministic clocks make async/database code testable.

## Wave 4: [stretch] review API and dashboard integration, only after T08 and FSRS foundation

- [ ] **T15 [stretch] [S] (30 min) Implement due-review query and sibling selection**
  - Files: `crates/server/src/reviews/query.rs`, `reviews/mod.rs`, and API tests.
  - Implement `GET /api/reviews/due` with ownership-first filtering, due ordering, a bounded result, and a published sibling problem that is not solved by the current user when one exists. Use the normal challenge fallback when no sibling exists and label it correctly.
  - Verify: two users with the same concept have independent due results; solved/unpublished problems are excluded; no arbitrary user id can affect the response.
  - Requirements: [stretch] FSRS-3.1 and FSRS-3.2.
  - Depends on: T12, T13, T14, and catalogue publication/solved seams.
  - Rust lesson: explain why ownership filters must happen before joins and how `Option` represents “no due card” without inventing one.

- [ ] **T16 [stretch] [S] (30 min) Implement atomic review grading** [SUPERVISED]
  - Files: `crates/server/src/reviews/handler.rs`, `reviews/query.rs`, and review migration/tests as needed.
  - Implement `POST /api/reviews/{concept}` with authenticated ownership, grade/version validation, row lock or safe initial-card creation, FSRS calculation, invariant checks, card update, history insert, sibling selection, and one transaction.
  - Return `409 review_conflict` for stale versions and leave both card and history unchanged. Return safe `scheduler_invariant_failed` when the adapter rejects a library result.
  - Verify: concurrent/stale writes cannot overwrite a newer card; successful writes create exactly one history row; failed transactions create no history and no partial card update.
  - Requirements: [stretch] FSRS-1, FSRS-2, FSRS-3, and DASH-8.
  - Depends on: T12 through T15.
  - Rust lesson: explain mutable borrowing through a transaction, row locks as database ownership, and why `?` causes the whole transaction to roll back on failure.

- [ ] **T17 [stretch] [P] (25 min) Add due-review dashboard integration**
  - Files: `crates/server/src/dashboard/{types.rs,query.rs,handler.rs}`, `web/src/lib/dashboard/`, and dashboard route components.
  - Add the review summary as an additive field only after the review tables and endpoint work. Keep core fields unchanged. Show due concept and sibling challenge with a clear `Spaced repetition` label; show null/empty state when no card is due.
  - Verify: the core dashboard still works when FSRS is disabled or there are no cards; due data is owned by the current user; the UI distinguishes a review from a normal suggestion.
  - Requirements: [stretch] FSRS-4 and DASH-9.
  - Depends on: T15, T16, and T10.
  - Rust lesson: explain additive API evolution and how optional Rust fields avoid fabricating data while keeping older clients compatible.

- [ ] **T18 [stretch] [S] Run the spaced-review verification gate**
  - Files: only fixes required by verification.
  - Run FSRS property tests, ignored PostgreSQL card/history/ownership/concurrency tests, server Clippy/tests, frontend check/build, `cargo deny check licenses`, and migration checks. Confirm no task uses `rs-fsrs`, Valkey as truth, or user code execution.
  - Verify: interval properties, dashboard privacy, stale-write rollback, due ordering, and reduced-motion/accessible review UI all pass. If this gate fails, disable/cut the stretch slice without regressing the core dashboard.
  - Requirements: all [stretch] requirements.
  - Depends on: T11 through T17.
  - Rust lesson: explain how the verification gate combines pure properties with database invariants and why a failed stretch gate must not weaken a shipped Must feature.

## Dependency summary

```text
Existing auth + run-lifecycle + catalogue seams
                  │
        T01 ──────┼────── T02
          │       │        │
          ├── T04 ┘        │
          └── T03 ─────────┘
                  │
             T05 API
             ├── T06 privacy/DB tests
             └── T07 API tests
                  │
             T08 CORE GATE
                  │
             T09 frontend ── T10 core smoke
                  │
        [stretch gate opens only here]
                  │
        T11 ── T12 ── T13 ── T14
                          │
                         T15 ── T16 ── T17 ── T18
```

T02 may be omitted if existing indexes already prove the required access paths, but the implementation task must record that inspection. T06 and T07 may run in parallel after the query and handler exist. T09 may start after the API contract is stable, but T10 and T08 are the core completion gates. No [stretch] task may be marked in progress before T08 passes.

## Completion checklist

### Must: core dashboard

- [ ] `GET /api/dashboard` requires the authenticated session and accepts no owner selector.
- [ ] One PostgreSQL read produces solved counts, streak, weak concepts, recent submissions, and suggestions.
- [ ] Solved problems are distinct per user and accepted verdicts only.
- [ ] Streak uses completed activity days and deterministic UTC boundaries.
- [ ] Weak concepts and suggestions use only the signed-in user's progress plus public catalogue data.
- [ ] Recent submissions are capped at ten and contain no code or hidden judge data.
- [ ] Suggestions are capped at three, deterministic, published, and unsolved for that user.
- [ ] Cross-user canary tests prove no aggregate or recommendation leakage.
- [ ] Frontend loading, empty, error, signed-out, keyboard, semantic-token, and reduced-motion requirements pass.
- [ ] Core CI-equivalent checks and the freeze-safe checkpoint pass.

### [stretch]: spaced repetition

- [ ] `fsrs` is used and `rs-fsrs` is absent.
- [ ] Card/history migrations enforce ownership and non-negative persisted values.
- [ ] Property tests prove non-negative intervals, due-after-review, deterministic calculations, and the documented growth-after-good invariant.
- [ ] Due and grade endpoints use session ownership and optimistic concurrency.
- [ ] A review grade atomically updates the card and history or changes neither.
- [ ] Due reviews select a fresh published sibling when possible.
- [ ] Dashboard review data is additive, labelled, and never fabricated.
- [ ] Stretch verification passes without regressing the Must dashboard.
