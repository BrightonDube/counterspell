# Tasks: auth

All tasks are implementation tasks for the design in `design.md` and the requirements in `requirements.md`. No task in this document is executed by the spec-generation run. Each task is sized for 30 minutes or less. `[P]` means it can run in parallel with other tasks in the same wave once its dependencies are done. `[S]` means it is sequential. `[SUPERVISED]` means Brighton reviews the diff before the next task starts (tokens, sessions, the OAuth callback, the decision functions, privileges and the demo reset are the security-critical parts).

Priority labels in task titles: **Must**, **Should**, **Could**. If time runs short, cut in this order: T36 (delete account), T25 (register and reset), T23 (link email), T22 (link provider), T28 (demo daily reset, keep the seed). Never cut: sessions, magic link, OAuth sign-in, the no-linking rule, the demo account, CSRF, rate limits.

Conventions for every Rust task:

- Follow `.kiro/steering/rust-teaching.md`, `security.md`, `api-conventions.md` and `structure.md`. No `unwrap()` or `expect()` outside tests; no `unsafe` (the workspace forbids it).
- The workspace has Clippy pedantic on and CI runs `-D warnings`. Run `cargo clippy -p counterspell-server --all-targets -- -D warnings` before finishing.
- Every public item gets a plain-English doc comment.
- Use runtime-checked `sqlx::query(...)` / `query_as(...)`, not the `query!` macros, so building never needs a database.
- Name every database-backed test with a `pg_` prefix and mark it `#[ignore = "needs DATABASE_URL"]`. Valkey-backed tests use `valkey_` and `VALKEY_URL`. Use `#[sqlx::test(migrations = "../../migrations")]`. To run them locally, start a throwaway Postgres 17 and Valkey, export `DATABASE_URL` and `VALKEY_URL` in your shell, and run `cargo test -p counterspell-server -- --ignored pg_`. Never commit a URL or password.
- **Never read, write or create `.env` or any key or credential file.** Tests inject configuration through `AuthConfig::from_map`, never through the process environment. No test calls GitHub, Google or Resend.
- Valkey holds counters only. Nothing may keep a session, token or flow in Valkey or in process memory as the source of truth.
- Do not add a Cargo package. Do not add a dependency other than those named in T01. If `cargo deny check licenses` fails, stop and ask Brighton before touching `deny.toml`.
- End each Rust task with a **Rust lesson** of 3–5 lines appended to `docs/kiro-log/rust-lessons.md` in the existing format (`# Rust lesson: auth Txx`). The suggested topic is listed per task.
- Property tests use `proptest` and keep their regression files in the repository. Property IDs (P1 to P22) refer to `design.md`.
- All new server code lives under `crates/server/src/auth/` unless a task says otherwise. The server crate is a library plus a binary after `run-lifecycle`; T02 checks this.

## Wave 0: foundations (no database yet)

- [ ] **T01 [S] (25 min) Add dependencies and notices**
  - Files: `crates/server/Cargo.toml`, `THIRD-PARTY-NOTICES.md`, `Cargo.lock`.
  - Add to `counterspell-server` (only those that are not already present from `run-lifecycle`): `sqlx` (Postgres, Tokio runtime with rustls not needed, `uuid`, `time`, `json`, `migrate`), `axum-extra` (feature `cookie`), `argon2`, `sha2`, `hmac`, `rand`, `base64`, `hex`, `subtle`, `time`, `url`, `percent-encoding`, `oauth2`, `reqwest` (`default-features = false`, features `json`, `rustls-tls`), `async-trait`, `serde`, `serde_json`, `uuid` (`v4`, `serde`), `tracing`, plus whatever Valkey client `run-lifecycle` chose. Dev-dependencies: `proptest`, `wiremock`, `tower` (feature `util`), `tracing-subscriber`.
  - Add one line per crate to `THIRD-PARTY-NOTICES.md` (name, licence).
  - Requirements: scope constraints, R21.5.
  - Depends on: none.
  - Verify: `cargo check -p counterspell-server`; `cargo tree -p counterspell-server` shows no `bollard` and no `counterspell-judge`; `cargo tree -p counterspell-common` unchanged; `cargo deny check licenses` passes. TLS crates such as `ring` may need a licence clarification in `deny.toml`: if so, stop and ask Brighton.
  - Preserve: no new Cargo package; `counterspell-common` gains no auth dependency.
  - Rust lesson: Cargo features and why `default-features = false` keeps the dependency tree small.

- [x] **T02 [S] (30 min) Auth module skeleton: config, clock, errors**
  - Files: `crates/server/src/lib.rs` (create only if `run-lifecycle` has not), `crates/server/src/auth/{mod.rs,config.rs,clock.rs,errors.rs}`, `crates/common/src/ids.rs` (only if `UserId` does not exist yet).
  - `Redacted<T>` (Debug and Display print `[redacted]`). `AuthConfig::from_map(&HashMap<String,String>) -> Result<AuthConfig, ConfigError>` with every rule of R2 (required variables, provider enabled only with both values, production guards, development exceptions, demo needs both variables and a 16-character password, secret length, URL parsing). `from_env()` only builds the map. `Clock` trait with `SystemClock` and `ManualClock` (feature `testkit`). `AuthError` with `status()`, `code()`, `message()` mapping to the table in `design.md` and `IntoResponse` producing `{"code","message"}`.
  - Tests: table-driven unit tests for each config rule (missing `SESSION_SECRET` names the variable, not the value; production + `http` is refused; `log` transport refused in production; development + localhost allowed); `Debug` output of a config containing secrets never contains them.
  - Requirements: R2.1–R2.8, R19.4.
  - Depends on: T01.
  - Verify: `cargo test -p counterspell-server auth::config`; `cargo clippy -p counterspell-server --all-targets -- -D warnings`.
  - Preserve: no secret in `Debug`/`Display`; URLs derive only from `PUBLIC_BASE_URL`.
  - Rust lesson: newtypes (`Redacted`) and implementing `Debug` by hand.

- [x] **T03 [P] (25 min) `EmailAddress` (property P15)**
  - Files: `crates/server/src/auth/email.rs` (type and parser only; senders come in T14).
  - `EmailAddress::parse` per R4.3 and R4.4: trim, lower-case, ASCII, at most 254, one `@`, non-empty parts, dotted domain without empty labels, none of whitespace, control characters, `<>,;"`. No dot or `+tag` canonicalisation.
  - Tests (proptest): idempotent parse; accepted values satisfy every rule; a corpus (`a@b.co`, `A@B.CO`, `a+x@b.co`, `a@b`, `a@@b.co`, `a b@c.co`, `a@b.co\r\nBcc: x@y.z`, `"a"@b.co`, `a@b.co,c@d.co`, non-ASCII) is classified as expected; `a.b@x.co` and `ab@x.co` stay different.
  - Requirements: R4.3, R4.4.
  - Depends on: T02.
  - Verify: `cargo test -p counterspell-server auth::email`.
  - Preserve: nothing outside the parser may build an address string for mail.
  - Rust lesson: parse, don't validate; a type that can only hold valid values.

- [x] **T04 [P] (20 min) `return_to` sanitiser (property P16)**
  - Files: `crates/server/src/auth/return_to.rs`.
  - `sanitize_return_to(Option<&str>) -> String` per R6.8 (single leading `/`, not `//` or `/\`, no control characters, at most 200 characters, and the percent-decoded form obeys the same rules). Default `/`.
  - Tests (proptest): for random strings plus a corpus (`//evil.com`, `/\evil.com`, `/%2f%2fevil.com`, `/%5cevil.com`, `https://x`, `javascript:x`, `/ok?x=%20`, control characters, 201 characters), the result is `/` or a path whose `Url::join` with `https://counterspell.example` keeps the origin.
  - Requirements: R6.8, R5.9.
  - Depends on: T02.
  - Verify: `cargo test -p counterspell-server auth::return_to`.
  - Preserve: the value is stored server-side and never echoed into a header unchecked.
  - Rust lesson: `&str` versus `String` and returning owned data from a borrowed input.

- [x] **T05 [P] (30 min) Token primitives and the pure expiry model (part of P2, P4)** [SUPERVISED]
  - Files: `crates/server/src/auth/tokens.rs` (types and pure functions only; the SQL store is T13).
  - `RawToken::generate()` (32 bytes from `OsRng`, unpadded base64url; no `Debug` that prints the value), `TokenHash::of(&str)` (SHA-256), `Purpose` enum with `as_str` and `parse`, constants `EMAIL_TOKEN_TTL = 15 min`, `FLOW_TTL = 10 min`, and `token_status(issued, expires, used, now) -> Valid | Expired | Used` as the reference model used by property tests.
  - Tests: generated tokens are 43 characters, unique across 10 000 draws, base64url only; hashing is deterministic; `token_status` is `Valid` iff `used.is_none() && issued <= now < expires`; `expires - issued` is exactly 15 minutes for tokens made by the issue helper.
  - Requirements: R3.1–R3.3, R3.8.
  - Depends on: T02.
  - Verify: `cargo test -p counterspell-server auth::tokens`.
  - Preserve: raw tokens never implement `Debug`/`Display` with the value.
  - Rust lesson: ownership of secrets and why we avoid `Clone` on them.

- [x] **T06 [P] (30 min) Pure decisions and capabilities (properties P5, P8 pure part)** [SUPERVISED]
  - Files: `crates/server/src/auth/decision.rs`, `crates/server/src/auth/capabilities.rs`.
  - `ProfileEmail`, `OAuthDecision`, `decide_oauth_login`; `Actor`, `LinkDecision`, `decide_link`; `Capabilities::for_user(is_demo)` and `require(Capability)`. `match` expressions with no wildcard arm.
  - Tests (proptest, exhaustive combinations): `decide_oauth_login(Some(u), _) == SignIn(u)` for every email state; with `None` and `Taken` the result is `RefuseEmailTaken`; `SignIn` only ever carries the identity owner; `decide_link` order (demo, same owner, other owner, provider already linked, link); `Capabilities::for_user(true)` is all false and `for_user(false)` all true.
  - Requirements: R7.1–R7.5, R8.4, R8.5, R16.4.
  - Depends on: T02.
  - Verify: `cargo test -p counterspell-server auth::decision auth::capabilities`.
  - Preserve: no database or I/O in these files; no function that looks up an account by email.
  - Rust lesson: enums that make forbidden states unrepresentable.

## Wave 1: schema

- [x] **T07 [S] (30 min) Migration: core auth tables** [SUPERVISED]
  - Files: `migrations/<timestamp>_auth_core.sql` (timestamp after the `run-lifecycle` migrations), `crates/server/tests/pg_auth_schema.rs`.
  - `CREATE EXTENSION IF NOT EXISTS citext`; tables `users`, `identities`, `email_tokens`, `sessions`, `oauth_flows` with every column, `CHECK`, index and the `sessions` update trigger exactly as in `design.md`. No grants yet.
  - Tests (`pg_`): an unverified email is refused (`users_email_is_verified`); a password without an email is refused; a second demo user is refused; a 16-minute `email_tokens` row is refused; an 11-minute flow is refused; a `login` flow with a session id and a `link` flow without one are refused; changing `sessions.user_id` or lowering `expires_at` is refused; `absolute_expires_at` beyond 90 days is refused; deleting a user cascades to identities, sessions, tokens and flows; deleting a session cascades to its `link` flows; two identities of one provider for one user are refused.
  - Requirements: R18.1–R18.5.
  - Depends on: none.
  - Verify: `cargo test -p counterspell-server -- --ignored pg_auth_schema`; run `sqlx migrate run` twice against a throwaway database; the second run is a no-op.
  - Preserve: no secret and no seed in the migration.
  - Rust lesson: none (SQL only); note what a `CHECK` constraint buys over application code.

- [ ] **T08 [S] (30 min) Migration: privileges, `submissions` foreign key, cascade rule (property P20)** [SUPERVISED]
  - Files: `migrations/<timestamp>_auth_privileges.sql`, `migrations/<timestamp>_auth_submissions_fk.sql`, `crates/server/tests/pg_auth_privileges.rs`.
  - Idempotently ensure the `NOLOGIN` roles `counterspell_server` and `counterspell_judge` exist (same `DO` block pattern as `run-lifecycle`). `REVOKE ALL … FROM PUBLIC` on the five tables, then the grants in the privilege matrix of `design.md`. Guarded `ALTER TABLE submissions ADD CONSTRAINT submissions_user_fk … ON DELETE CASCADE` (skip when `submissions` does not exist).
  - Tests (`pg_`): `has_table_privilege` is false for `counterspell_judge` and `PUBLIC` on all five tables for every privilege; the server role has no `TRUNCATE` and no `UPDATE` on `handle`, `is_demo`, `demo_reset_at`; `pg_all_fks_to_users_and_submissions_cascade` asserts `confdeltype = 'c'` for every foreign key whose target is `users` or `submissions`; when `submissions` exists, inserting a submission for an unknown user fails.
  - Requirements: R1.3, R18.6–R18.8.
  - Depends on: T07.
  - Verify: `cargo test -p counterspell-server -- --ignored pg_auth_privileges`.
  - Preserve: the judge role gets nothing; Postgres stays the only store.
  - Rust lesson: none (SQL only); note least privilege.

- [ ] **T09 [S] (30 min) Migration: demo guards, seed helper and reset function (properties P17 database part, P18 prerequisite)** [SUPERVISED]
  - Files: `migrations/<timestamp>_auth_demo.sql`, `crates/server/tests/pg_auth_demo_db.rs`.
  - `cs_guard_demo_user()` trigger on `users` (insert, update, delete), the `identities` before-insert trigger that refuses the demo user, `cs_seed_demo_user(text, text)` (`SECURITY DEFINER`, fixed `search_path`, sets `counterspell.demo_seed` locally, upserts the single demo user), `cs_reset_demo_account()` (as in `design.md`, including the advisory lock and the generic per-user-table loop), grants and revokes for both functions.
  - Tests (`pg_`): without the setting, inserting a demo user, deleting the demo user, changing its email or password, and inserting an identity for it all fail; `cs_seed_demo_user` creates exactly one demo user and is idempotent; the server role can execute the functions and the judge role cannot; the reset function removes only the demo user's finished rows (use a temporary table with a cascading foreign key to `users` to prove the generic loop), keeps running rows, resets counters, sets `demo_reset_at`, and leaves a second user's rows byte-identical.
  - Requirements: R16.6–R16.8, R18.8.
  - Depends on: T07, T08.
  - Verify: `cargo test -p counterspell-server -- --ignored pg_auth_demo_db`.
  - Preserve: the functions can act only on the demo user; no user text reaches dynamic SQL.
  - Rust lesson: none (SQL only); note `SECURITY DEFINER` and why `search_path` is fixed.

## Wave 2: sessions

- [ ] **T10 [S] (30 min) `SessionService` (parts of P4, P11, P12)** [SUPERVISED]
  - Files: `crates/server/src/auth/session.rs`, `crates/server/tests/pg_auth_session.rs`.
  - `SessionKey` (SHA-256 hex), `SessionService::{create, authenticate, revoke, revoke_all, is_valid}` with the SQL of `design.md`, the 30-day rolling and 90-day absolute limits, the one-hour renewal throttle, and `Clock` injection. `create` returns the raw cookie value once.
  - Tests (`pg_`, `ManualClock`): created sessions store only the hash; a session authenticates iff `now < min(last_renewed + 30 d, created + 90 d)`; renewal happens at most once per hour and never passes the absolute limit; unknown, expired and wrong-case values do not authenticate; `revoke_all` deletes exactly one user's sessions; a session of a deleted user is gone.
  - Requirements: R10.2–R10.5, R10.7–R10.9, R11.2, R11.6.
  - Depends on: T02, T05, T07.
  - Verify: `cargo test -p counterspell-server -- --ignored pg_auth_session`.
  - Preserve: the database is authoritative for expiry, never the cookie's `Max-Age`.
  - Rust lesson: `async fn`, `Result`, and the `?` operator with a custom error type.

- [ ] **T11 [S] (30 min) Cookie handling, extractors and the `UserResolver` seam** [SUPERVISED]
  - Files: `crates/server/src/auth/{session.rs,resolver.rs,mod.rs}`, `crates/server/src/auth/handlers/{session.rs,me.rs}`, `crates/server/src/seams.rs` (only if the trait signature must change), `crates/server/tests/auth_session_http.rs`.
  - Cookie builder (names, attributes and dev exceptions from R2.6 and `design.md`), `CookieSlot` middleware `session_cookie`, extractors `AuthUser` and `MaybeAuthUser` (take no input from the request except the cookie), `SessionUserResolver` implementing `UserResolver`, handlers `POST /api/auth/logout`, `POST /api/auth/logout-all` (capability check), `GET /api/me` (no-store, caller's data only), and an `auth::router(state)` skeleton that mounts them.
  - Tests: no cookie, garbage cookie and expired cookie give `401 unauthenticated` and a clearing cookie when the value was unusable; `GET /api/me` returns only the caller; `logout` is idempotent; `logout-all` kills all of the user's sessions and no one else's; `SessionUserResolver` returns the same user as the extractor.
  - Requirements: R10.1, R10.4–R10.5, R11.1–R11.4, R12.1–R12.4.
  - Depends on: T10, T06, T08.
  - Verify: `cargo test -p counterspell-server auth_session_http -- --include-ignored`.
  - Preserve: there is no code path that reads a user id from a body, path, query or other header.
  - Rust lesson: Axum extractors and why handlers get the user from a type, not a parameter.

- [ ] **T12 [S] (30 min) Session property tests (P9, P10, P11, P12, P13)** [SUPERVISED]
  - Files: `crates/server/tests/pg_auth_session_props.rs`, `proptest-regressions/`.
  - Stateful model: random users and operations (create, authenticate, renew via manual clock, logout, logout-all, `/api/me`, resolver) against a pure model; assert isolation (A's session never returns B's data), logout and logout-all effects, lifetime rules, a new session value on every `create`, and that every `Set-Cookie` produced by the session handlers has the required attributes.
  - Requirements: R10.1, R10.3–R10.6, R11, R12.1, R12.5.
  - Depends on: T11.
  - Verify: `cargo test -p counterspell-server -- --ignored pg_auth_session_props` with `PROPTEST_CASES=256`.
  - Preserve: failures print the shrunk operation sequence.
  - Rust lesson: property tests and shrinking.

## Wave 3: tokens, mail, limits, request hardening, magic link

- [ ] **T13 [S] (30 min) `TokenStore` (properties P1, P2, P3, P4)** [SUPERVISED]
  - Files: `crates/server/src/auth/tokens.rs`, `crates/server/tests/pg_auth_tokens.rs`.
  - `TokenStore::{issue, peek, consume}` with the exact SQL of `design.md` (supersede then insert in one transaction; one-statement consume with purpose, expiry, used and bound-user checks). Any failure returns `InvalidOrExpired`.
  - Tests (`pg_`, `ManualClock`): 32 concurrent consumers of one token produce exactly one success (P1); consume at `t` succeeds iff the model says valid, including the exact boundary at 15 minutes (P2); a token never validates for another purpose; issuing again supersedes the earlier token and leaves one live token per (email, purpose) (P3); after random operations no table contains a raw token (P4); `link_email` with the wrong bound user fails without marking the token used.
  - Requirements: R3.1–R3.8, R9.3.
  - Depends on: T05, T07.
  - Verify: `cargo test -p counterspell-server -- --ignored pg_auth_tokens` with `PROPTEST_CASES=256`.
  - Preserve: expiry uses the injected Clock; consume is one atomic statement, never read-then-write.
  - Rust lesson: sharing a pool across tasks (`Arc`, `Clone` on `PgPool`) and `tokio::spawn`.

- [ ] **T14 [S] (30 min) Mail senders and templates (parts of P14, P15)**
  - Files: `crates/server/src/auth/email.rs`, `crates/server/tests/auth_mail.rs`.
  - `EmailSender` trait, `OutgoingMail`, `ResendSender` (reqwest, bearer key, body `{from,to:[to],subject,html,text}`, `Idempotency-Key`, 3 s / 10 s timeouts, one retry on connect error or `5xx`), `MemorySender`, `LogSender` (refused when `APP_ENV=production`), `GuardedSender` (refuses the demo address; applies the global daily cap through the limiter trait from T15 once available, otherwise a stub), fixed templates per purpose with HTML escaping, link builder `{PUBLIC_BASE_URL}/auth/verify#token=…`.
  - Tests: wiremock checks headers, JSON shape, one recipient, idempotency key, retry on `500` once, no retry on `422`; templates escape `<script>` in values; the link uses the fragment and the configured origin only; the demo address gets no mail.
  - Requirements: R4.1, R4.2, R4.5–R4.7, R4.9.
  - Depends on: T02, T03.
  - Verify: `cargo test -p counterspell-server auth_mail`.
  - Preserve: the Resend key is never logged; nothing builds a link from a request header.
  - Rust lesson: traits and `Arc<dyn Trait>` for swappable implementations.

- [ ] **T15 [S] (30 min) Auth rate limits and client IP (property P19)**
  - Files: `crates/server/src/auth/limits.rs`, and `crates/server/src/{net.rs,limits.rs}` only if `run-lifecycle` has not created `ClientIp` and the fixed-window counter.
  - `AuthLimiter` over the shared window counter: actions, scopes and limits from `design.md`; keys `rl:auth:<action>:<scope>:<hmac_hex>` using HMAC-SHA256 with `SESSION_SECRET`; `Retry-After` computation; fail-closed for mail actions and the daily cap, fail-open (with a warning) for others; a store trait with an in-memory fake for tests.
  - Tests: request `limit + 1` is refused with `retry_after >= 1`; a refused request does not consume another unit; scopes are independent; keys contain no raw email or IP; Valkey down gives `EmailUnavailable` for mail actions and allows login attempts; daily cap blocks at 80 and resets the next UTC day. A `valkey_` test runs the same checks against real Valkey.
  - Requirements: R14.1–R14.4, R14.6, R14.7, R4.8.
  - Depends on: T02.
  - Verify: `cargo test -p counterspell-server auth::limits`; `VALKEY_URL=… cargo test -- --ignored valkey_auth`.
  - Preserve: counters only; no personal data in keys.
  - Rust lesson: HMAC keys, `&[u8]` versus `Vec<u8>`.

- [ ] **T16 [S] (25 min) CSRF origin guard and response layers (property P22)**
  - Files: `crates/server/src/auth/csrf.rs`, `crates/server/tests/auth_csrf.rs`.
  - `origin_guard` middleware (rule of R13.1), JSON-only and 16 KiB body layer for auth routes, `no_store` layer, `no_referrer` redirect layer. Expose a function `apply_api_hardening(Router) -> Router` that T33 applies to the whole `/api` router.
  - Tests: a table of (method, `Origin`, `Sec-Fetch-Site`, `Cookie`) combinations against the predicate (proptest); refused requests reach no handler (a counter handler proves it); `GET` is never blocked; wrong content type gives `415`; a 17 KiB body gives `413`.
  - Requirements: R13.1–R13.6.
  - Depends on: T02.
  - Verify: `cargo test -p counterspell-server auth_csrf`.
  - Preserve: the guard is global to `/api`, not per route.
  - Rust lesson: Axum middleware with `from_fn_with_state` and `Next`.

- [ ] **T17 [S] (30 min) Magic-link request and verify handlers (property P14)** [SUPERVISED]
  - Files: `crates/server/src/auth/handlers/{magic.rs,verify.rs}`, `crates/server/tests/auth_magic.rs`.
  - `POST /api/auth/email/magic` (parse, sanitise `return_to`, limiter, issue, detached send, uniform `202`), `POST /api/auth/email/verify` (`peek`, purpose dispatch, consume; for `magic_link`: find or create the user in one transaction with a generated handle, create a session, set cookie, return `{user, return_to}`). Other purposes are stubs returning `invalid_or_expired_token` until T23 and T25.
  - Tests (`MemorySender`, `ManualClock`, real Postgres): known and unknown addresses give identical responses; the link extracted from the sent mail signs in once and fails on the second use and after 15 minutes; two concurrent verifies create one user; the demo address gets a `202` and no mail, and its tokens are refused; a request cannot choose the account except through the token; `GET` on the verify path never consumes.
  - Requirements: R5.1–R5.9, R3.4, R3.5, R16.5.
  - Depends on: T10, T11, T13, T14, T15, T16.
  - Verify: `cargo test -p counterspell-server auth_magic -- --include-ignored`.
  - Preserve: the response never depends on whether an account exists.
  - Rust lesson: transactions in sqlx (`begin`, `commit`) and what happens on early return.

## Wave 4: OAuth sign-in

- [ ] **T18 [S] (30 min) Provider trait, fake provider and flow store (property P7)** [SUPERVISED]
  - Files: `crates/server/src/auth/oauth/{mod.rs,flow.rs,fake.rs}`, `crates/server/tests/pg_auth_flow.rs`.
  - `ProviderId`, `ProviderProfile`, `OAuthProvider` trait; `FakeProvider` (testkit) returning configured profiles per code; `FlowStore::{create_login, create_link, consume}` with PKCE via `oauth2::PkceCodeChallenge::new_random_sha256()`, hashed `state` and binding, 10-minute life, single-use `DELETE … RETURNING`.
  - Tests (`pg_`, `ManualClock`): the stored verifier hashes (S256) to the challenge in the authorize URL; a flow works once; wrong provider, altered `state`, expired flow, and (login) missing or wrong binding fail; a failed attempt still consumes the flow; link flows are bound to the session and are deleted when the session is deleted.
  - Requirements: R6.1, R6.3, R8.1, R8.2, R8.7.
  - Depends on: T02, T04, T05, T06, T07.
  - Verify: `cargo test -p counterspell-server -- --ignored pg_auth_flow`.
  - Preserve: the verifier and `state` are never logged; `redirect_uri` comes from configuration.
  - Rust lesson: `async_trait` and trait objects.

- [ ] **T19 [P] (30 min) GitHub adapter**
  - Files: `crates/server/src/auth/oauth/github.rs`, `crates/server/tests/auth_github.rs`.
  - Real `OAuthProvider` for GitHub using the `oauth2` crate for the code exchange with the PKCE verifier and `reqwest` for `GET /user` and `GET /user/emails`. Use the numeric `id` as the stable key; use only the primary, verified email; timeouts, 64 KiB response cap, no redirects on the token call. Pin the `oauth2` major version and adapt the API (marked **[U]** in the design).
  - Tests with `wiremock` standing in for GitHub (base URLs configurable): the token request carries `code_verifier`; profile requests carry the bearer token, `Accept` and `User-Agent`; an unverified or non-primary email yields `verified_email: None`; a `500` or malformed body yields a `ProviderError` without leaking the body.
  - Requirements: R6.2, R6.4, R6.6, R6.7, R6.9, R6.10.
  - Depends on: T18.
  - Verify: `cargo test -p counterspell-server auth_github`.
  - Preserve: the access token is dropped after use and never logged.
  - Rust lesson: lifetimes in builder-style APIs, in plain terms.

- [ ] **T20 [P] (30 min) Google adapter**
  - Files: `crates/server/src/auth/oauth/google.rs`, `crates/server/tests/auth_google.rs`.
  - Real `OAuthProvider` for Google: code exchange with the verifier, then `userinfo`. Use `sub` as the stable key; use the email only when `email_verified` is true. Same limits as T19.
  - Tests with `wiremock`: verifier sent; `email_verified: false` yields no email; missing `sub` is an error; error bodies are not echoed.
  - Requirements: R6.2, R6.4, R6.6, R6.7, R6.9, R6.10.
  - Depends on: T18.
  - Verify: `cargo test -p counterspell-server auth_google`.
  - Preserve: no ID-token parsing, so no JWT dependency.
  - Rust lesson: `serde` structs with optional fields and why unknown fields are ignored.

- [ ] **T21 [S] (30 min) OAuth start and callback for sign-in (properties P6, P7)** [SUPERVISED]
  - Files: `crates/server/src/auth/handlers/oauth.rs`, `crates/server/tests/pg_auth_oauth_model.rs`.
  - `GET …/start` (provider enabled, limiter, `create_login`, binding cookie, `303`) and `GET …/callback` for `login` flows: the order of checks of R6.3 (flow consumed first, binding checked before the code is spent), `fetch_profile`, the `ProfileEmail` lookup, `decide_oauth_login`, the transaction for `CreateAccount` with the retry rules of `design.md`, new session, cookie, `303 → return_to`; every failure redirects to `/login?error=<code>`. Callback never returns JSON. The `link` branch is a stub until T22.
  - Tests (`FakeProvider`, real Postgres): happy path creates one user and one identity; second sign-in reuses it; provider email equal to an existing account's email gives `account_exists`, no session, and every table unchanged (P6); the model test applies random sequences of callbacks with random profiles and asserts the invariants of P6; a replayed or foreign `state` fails; a callback without the binding cookie fails and still consumes the flow; `access_denied` redirects with `oauth_denied`; an unverified provider email creates a user without an email; the demo address is treated as taken.
  - Requirements: R6.1–R6.11, R7.1–R7.7, R16.5.
  - Depends on: T10, T11, T15, T16, T18, T06.
  - Verify: `cargo test -p counterspell-server pg_auth_oauth_model -- --include-ignored` with `PROPTEST_CASES=256`.
  - Preserve: nothing looks up an account by provider email except the single "taken" check.
  - Rust lesson: `match` on enums returned by pure functions and keeping side effects at the edges.

## Wave 5: user-initiated linking

- [ ] **T22 [S] (30 min) Link a provider identity (property P8)** [SUPERVISED]
  - Files: `crates/server/src/auth/handlers/link.rs`, `crates/server/src/auth/handlers/oauth.rs` (the `link` branch), `crates/server/tests/pg_auth_link_provider.rs`.
  - `POST /api/auth/link/{provider}/start` (session, capability, limiter, `create_link`, `200 {authorize_url}`) and the `link` branch of the callback (current session must equal the flow's session key and user, `fetch_profile`, `decide_link`, insert, redirect to `/settings/account?linked=` or `?error=`).
  - Tests: linking adds exactly one identity to the signed-in user; a callback made with another user's session or no session fails with `state_invalid`; logging out before the callback kills the flow; an identity owned by another user gives `identity_already_linked` and changes nothing; the same identity again is a no-op; a second identity for the same provider gives `provider_already_linked`; the provider's email is ignored (a different or colliding email changes nothing); the demo session is refused with `403`.
  - Requirements: R8.1–R8.8.
  - Depends on: T21, T06.
  - Verify: `cargo test -p counterspell-server pg_auth_link_provider -- --include-ignored`.
  - Preserve: an identity is never moved between accounts.
  - Rust lesson: borrowing a request-scoped value across `.await` points.

- [ ] **T23 [S] (30 min) Link an email address** [SUPERVISED]
  - Files: `crates/server/src/auth/handlers/link.rs`, `crates/server/src/auth/handlers/verify.rs` (the `link_email` branch), `crates/server/tests/pg_auth_link_email.rs`.
  - `POST /api/auth/link/email` (session, capability, `409 email_already_set`, limiter, uniform `202`, no mail when the address belongs to another account) and the verify branch: `peek` then `MaybeAuthUser`; no session → `401`; other user → `invalid_or_expired_token` without consuming; same user → consume with the bound user, `UPDATE users SET email …, email_verified_at … WHERE id = $u AND email IS NULL`; unique violation → `409 email_in_use`. No session is created or changed.
  - Tests: full happy path through `MemorySender`; response identical for free and taken addresses (P14); wrong-session attempt does not burn the token and the right session still works; second use fails; expiry at 15 minutes; demo refused; a user who already has an email gets `409`.
  - Requirements: R9.1–R9.6, R14.1.
  - Depends on: T13, T14, T17, T06.
  - Verify: `cargo test -p counterspell-server pg_auth_link_email -- --include-ignored`.
  - Preserve: a verification link can attach an address only to the account it was issued for.
  - Rust lesson: `Option` combinators instead of nested `if let`.

## Wave 6: passwords

- [ ] **T24 [S] (30 min) Password hashing and login (properties P4 hash part, P14 login part)** [SUPERVISED]
  - Files: `crates/server/src/auth/password.rs`, `crates/server/src/auth/handlers/login.rs`, `crates/server/tests/pg_auth_login.rs`.
  - Argon2id (19 456 KiB, 2, 1) with PHC strings, `verify` → `Verified | NeedsRehash | Rejected`, `verify_dummy`, `HashGate` (4 permits, 2 s wait), policy check (10–128 characters, ≤1024 bytes), `spawn_blocking`. `POST /api/auth/login` with the limiter (read failures first, count failures), uniform `401 invalid_credentials`, rehash on weak parameters, session creation.
  - Tests: round trip and wrong password; PHC parameters at or above the minimum; unknown email, wrong password and passwordless account give byte-identical responses; five failures for one (email, IP) give `429` before any hashing (a counter on the hasher proves it); a saturated gate gives `503 busy`; the password never appears in captured logs.
  - Requirements: R15.1–R15.6, R14.1, R14.2, R14.5.
  - Depends on: T10, T11, T15, T16.
  - Verify: `cargo test -p counterspell-server pg_auth_login -- --include-ignored`.
  - Preserve: hashing never runs on async worker threads.
  - Rust lesson: `spawn_blocking` and why CPU-heavy work must leave the async runtime.

- [ ] **T25 [S] (30 min) Register and password reset (Should)** [SUPERVISED]
  - Files: `crates/server/src/auth/handlers/{register.rs,reset.rs,verify.rs}`, `crates/server/tests/pg_auth_register_reset.rs`.
  - `POST /api/auth/register` (policy, limiter, hash through the gate, `issue(Verify, …, {"password_hash","return_to"})`, uniform `202`), the `verify` branch (create the user and a `register` session, or `409 account_exists` with no change and no session), `POST /api/auth/password/reset` (uniform `202`; mail only for an existing, passworded, non-demo account), `POST /api/auth/password/reset/confirm` (consume `Reset`, policy, update the hash, `revoke_all`, `204`, no sign-in).
  - Tests: no `users` row exists before verification; a mailbox owner registering an address that has an account changes nothing and signs in nobody; responses are identical for known and unknown addresses (P14); reset revokes every session of the user; reset for the demo or for a passwordless account sends nothing; a password can only be set by these two paths (assert over all handlers).
  - Requirements: R15.7–R15.9, R11.5, R16.4, R16.5.
  - Depends on: T24, T13, T14, T17.
  - Verify: `cargo test -p counterspell-server pg_auth_register_reset -- --include-ignored`.
  - Preserve: no account row without mailbox proof; no password added to an existing passwordless account.
  - Rust lesson: sharing code between handlers with small helper functions instead of macros.

## Wave 7: demo account

- [ ] **T26 [S] (25 min) Demo seed and `GET /api/auth/config` (Must)**
  - Files: `crates/server/src/auth/demo.rs`, `crates/server/src/auth/handlers/config.rs`, `crates/server/tests/pg_auth_demo_seed.rs`.
  - `seed_demo(pool, &DemoConfig, hasher)`: hash only if the stored hash does not already verify the password; call `cs_seed_demo_user`. Called from start-up code (wired in T33). `GET /api/auth/config` as in `design.md` (no secrets, no password).
  - Tests: seeding twice leaves exactly one demo user with `is_demo = true`, handle `demo`, verified email; the demo user can log in with the configured password via T24's handler; with the variables unset nothing is seeded and `config` reports `demo.enabled = false`; the stored value is an Argon2id hash and the plain password appears nowhere in the database or in captured logs.
  - Requirements: R16.1–R16.3, R16.9.
  - Depends on: T09, T24.
  - Verify: `cargo test -p counterspell-server pg_auth_demo_seed -- --include-ignored`.
  - Preserve: the password comes only from the environment through `AuthConfig`; no file is read.
  - Rust lesson: `Option<Config>` as "feature on or off".

- [ ] **T27 [S] (30 min) Enforce capabilities everywhere (property P17)** [SUPERVISED]
  - Files: `crates/server/src/auth/handlers/*.rs` (call sites only), `crates/server/tests/pg_auth_demo_matrix.rs`.
  - Make every mutating handler call `Capabilities::require` first (link provider start, link email, register-verify and reset paths for the demo address, `logout-all`, delete when it exists). Add a test that enumerates the auth router's mutating routes and fails when one does not appear in the matrix.
  - Tests: a demo session against every mutating endpoint gives `403 demo_account_restricted` (or the documented refusal) and the database snapshot is unchanged; `logout` works; magic link, tokens and OAuth never reach the demo user; the database triggers still refuse direct changes.
  - Requirements: R16.4–R16.6.
  - Depends on: T22, T23, T25, T26.
  - Verify: `cargo test -p counterspell-server pg_auth_demo_matrix -- --include-ignored`.
  - Preserve: the demo's limits live in `Capabilities` only; no scattered `if user.is_demo`.
  - Rust lesson: exhaustive tests that fail when someone adds a route.

- [ ] **T28 [S] (30 min) Daily demo reset (Should; property P18)** [SUPERVISED]
  - Files: `crates/server/src/auth/demo.rs`, `crates/server/tests/pg_auth_demo_reset.rs`.
  - A background task that every 5 minutes computes the latest scheduled reset (`DEMO_RESET_HOUR_UTC`), reads `users.demo_reset_at`, and calls `cs_reset_demo_account()` when it is older; catches up on start-up; logs the count. Takes the `Clock`; cancellable for tests and shutdown.
  - Tests (`ManualClock`): no reset before the hour; one reset after it; a restart after a missed hour resets once; two concurrent calls reset once; other users' rows are unchanged (snapshot comparison); running submissions survive; sessions of the demo user survive.
  - Requirements: R16.7, R16.8.
  - Depends on: T09, T26.
  - Verify: `cargo test -p counterspell-server pg_auth_demo_reset -- --include-ignored`.
  - Preserve: the task never deletes with its own SQL; it only calls the database function.
  - Rust lesson: background tasks with `tokio::spawn`, `select!` and graceful cancellation.

## Wave 8: web screens

- [ ] **T29 [P] (25 min) Auth client and session store**
  - Files: `web/src/lib/auth/client.js`, `web/src/lib/auth/session.svelte.js`, `web/src/lib/auth/messages.js`.
  - Fetch wrapper (same-origin, JSON, maps `{code,message}` errors), calls for every endpoint in the API table, a Svelte 5 rune store holding `user | null` and `capabilities`, a `returnToFromQuery()` that applies the same rules as the server (convenience only), a fixed table of `error` code → plain sentence.
  - Requirements: R20.5–R20.7.
  - Depends on: none.
  - Verify: `cd web && npm run check && npm run build`.
  - Preserve: no token or password is stored in `localStorage`, `sessionStorage` or a global; messages are rendered as text.

- [ ] **T30 [S] (30 min) Login page**
  - Files: `web/src/routes/login/+page.svelte` (and small components under `web/src/lib/auth/`).
  - Provider buttons for enabled providers (plain links), email form with magic-link and password modes (Bits UI tabs), the "Judging? Use the demo account" link that opens the demo form (demo email shown, empty password field), `aria-live` error banner for `?error=`, labelled inputs, visible focus. Semantic tokens only.
  - Requirements: R20.1, R20.4, R20.5, R16.3.
  - Depends on: T29.
  - Verify: `cd web && npm run check && npm run build`; manual keyboard-only pass; no raw hex colours (`grep` the new files).
  - Preserve: nothing fabricated; the demo password is never prefilled.

- [ ] **T31 [S] (25 min) Verify page**
  - Files: `web/src/routes/auth/verify/+page.svelte`.
  - Reads the token from the fragment, calls `history.replaceState` immediately, keeps the token in memory only, shows a confirm button, posts to `/api/auth/email/verify`, shows success (redirect to the validated `return_to`), "link used or expired" with a request-new-link form, and a "sign in first" message for the `401` of a link-email token.
  - Requirements: R20.2, R5.3, R9.3.
  - Depends on: T29.
  - Verify: `cd web && npm run check && npm run build`; manual: the address bar shows no token after load.
  - Preserve: no request is sent until the user presses the button.

- [ ] **T32 [S] (30 min) Account settings page**
  - Files: `web/src/routes/settings/account/+page.svelte`.
  - Shows email, linked providers, buttons driven by `capabilities` (link GitHub, link Google, add email, sign out everywhere, delete account when T36 exists), `?linked=` and `?error=` banners; for the demo account every action is disabled with visible text explaining why. `401` goes to `/login` with a sanitised `return_to`.
  - Requirements: R20.3, R20.4, R20.6, R16.9.
  - Depends on: T29.
  - Verify: `cd web && npm run check && npm run build`.
  - Preserve: the server decides what is allowed; the UI only reflects it.

## Wave 9: integration, hardening and finish

- [ ] **T33 [S] (30 min) Wire everything into the server, Compose and Caddy** [SUPERVISED]
  - Files: `crates/server/src/main.rs`, `crates/server/src/auth/{mod.rs,cleanup.rs}`, `docker-compose.yml`, `deploy/auth-setup.md`, `Caddyfile` (needs Brighton's approval for this file), `README.md` (one paragraph).
  - Build `AuthState` from `AuthConfig::from_env()`, mount `auth::router`, apply `apply_api_hardening` to the whole `/api` router, run `seed_demo` and the demo reset task, start the cleanup task (hourly delete of expired/used tokens and flows older than a day and expired sessions), choose real or log mail sender, register enabled providers. Compose: pass the variables of R2.1 as `${NAME}` pass-throughs to the `server` service only (never to `judge`), with `APP_ENV=development` and the `log` transport in the local override. `deploy/auth-setup.md`: OAuth app registration steps and the callback URLs `${PUBLIC_BASE_URL}/api/auth/oauth/<provider>/callback`, the Resend domain, and the demo account variables. Caddy: ask Brighton, then add a log filter that drops the query string for the OAuth callback path (open question 16).
  - Requirements: R1.4, R2, R3.9, R13.5, R19.3, R16.1.
  - Depends on: T16, T17, T19, T20, T21, T22, T23, T24, T26, T28.
  - Verify: `docker compose config` shows the new variables on `server` only; `cargo test --workspace`; start the stack in development mode, request `GET /api/auth/config`, request a magic link (the link appears in the dev log), sign in; `curl -X POST` without an `Origin` and with a cookie is refused.
  - Preserve: no `.env` is read or written by the task; `judge` receives no auth variable.
  - Rust lesson: structuring `main` so wiring is separate from logic.

- [ ] **T34 [S] (30 min) Connect to `run-lifecycle`: fixtures, stream revalidation, end-to-end**
  - Files: `crates/server/src/testkit.rs` (a `create_user()` helper), `crates/server/tests/pg_auth_runs_integration.rs`, the SSE handler of `run-lifecycle` (small edit), `docs/kiro-log/auth-integration.md`.
  - Give the `run-lifecycle` tests a real-user fixture now that `submissions.user_id` has a foreign key. Make the SSE loop call `SessionService::is_valid` on each heartbeat and close when it is false. Add tests: a session cookie authenticates `POST /api/runs`; a second user's cookie gets `404` for the first user's run and events; after `logout-all` the next request and the next heartbeat are refused; the production `UserResolver` is `SessionUserResolver`, not `Unauthenticated`.
  - Requirements: R11.6, R12.2, R12.5, R18.7.
  - Depends on: T33, and the implemented `run-lifecycle` endpoints.
  - Verify: `cargo test --workspace -- --include-ignored` passes including the `run-lifecycle` suite.
  - Preserve: ownership checks stay inside the `run-lifecycle` queries; auth only supplies `UserId`.

- [ ] **T35 [S] (25 min) Log-capture test (property P21)**
  - Files: `crates/server/tests/auth_logs.rs`.
  - Run every flow (magic link, OAuth sign-in and link, password login, register, reset, logout-all) with a capturing `tracing` subscriber and canary values for the token, session value, password, `code`, `state`, verifier, access token, client secret, Resend key and email. Assert none appears in any captured line, including the HTTP trace lines for the callback and verify routes (path only).
  - Requirements: R19.1–R19.4, R21.3.
  - Depends on: T33.
  - Verify: `cargo test -p counterspell-server auth_logs -- --include-ignored`.
  - Preserve: the test fails loudly if a new log line leaks a canary.

- [ ] **T36 [S] (30 min) Delete account (Could)**
  - Files: `crates/server/src/auth/handlers/delete.rs`, `crates/server/tests/pg_auth_delete.rs`; the settings page button (small edit to T32's file).
  - `DELETE /api/me {confirm_handle}`: capability, handle must match, `409 runs_active` when a non-terminal job exists, delete the user (cascade), delete unused email tokens for the address, clear the cookie, `204`.
  - Tests: everything of the user is gone (identities, sessions, tokens, flows, submissions and dependent rows); another user is untouched; wrong handle changes nothing; an active run blocks; the demo account is refused and the trigger would refuse it anyway.
  - Requirements: R17.1–R17.5.
  - Depends on: T33, T27.
  - Verify: `cargo test -p counterspell-server pg_auth_delete -- --include-ignored`.
  - Preserve: nothing is deleted without the handle confirmation.

- [ ] **T37 [S] (30 min) Final verification and the manual smoke checklist**
  - Files: no planned source changes; `docs/kiro-log/auth-dod.md` records results.
  - Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace -- --include-ignored` with `PROPTEST_CASES=512` for P1, P2, P3, P6, P9, P10, P11, P18, `cd web && npm run check && npm run build`, `cargo deny check licenses`, `docker compose config`. Then the manual checklist on the VPS with real services: GitHub sign-in, Google sign-in, magic link through Resend (SPF and DKIM pass), link GitHub to a magic-link account, link Google, add an email to an OAuth-only account, sign in with a Google account whose email matches an existing account and confirm `account_exists`, logout-all from a second browser, the demo account (restrictions visible), and the next-day demo reset. Record the results and any failure as a note, not as a silent pass.
  - Requirements: R21.1–R21.5.
  - Depends on: T35, T36 (or T36 recorded as cut).
  - Verify: every command passes; each property P1 to P22 has a passing test (list them in the record).
  - Preserve: do not start later features.

## Dependency summary and parallel waves

- **Wave 0:** T01, then T02. After T02, T03, T04, T05 and T06 can run in parallel.
- **Wave 1:** T07 can start any time (no Rust dependency). T08 follows T07. T09 follows T07 and T08.
- **Wave 2:** T10 follows T02, T05, T07. T11 follows T06, T08 and T10. T12 follows T11.
- **Wave 3:** T13 follows T05 and T07. T14 follows T02 and T03. T15 and T16 follow T02 and can run in parallel with T13 and T14. T17 follows T10, T11, T13, T14, T15, T16.
- **Wave 4:** T18 follows T02, T04, T05, T06, T07. T19 and T20 run in parallel after T18. T21 follows T10, T11, T15, T16, T18.
- **Wave 5:** T22 follows T21. T23 follows T13, T14, T17.
- **Wave 6:** T24 follows T10, T11, T15, T16. T25 follows T24, T13, T14, T17.
- **Wave 7:** T26 follows T09 and T24. T27 follows T22, T23, T25, T26. T28 follows T09 and T26.
- **Wave 8:** T29 has no dependency and can start at any time. T30, T31 and T32 follow T29 and run in parallel.
- **Wave 9:** T33 follows the server tasks above. T34 needs the implemented `run-lifecycle`. T35 and T36 follow T33. T37 is last.

## Completion checklist

- [ ] No new Cargo package was added; `cargo metadata --no-deps` still lists exactly the six canonical packages; the server's dependency tree has no `bollard` and no `counterspell-judge`.
- [ ] Postgres is the only store for users, identities, sessions, email tokens and OAuth flows; Valkey holds counters only.
- [ ] The judge role has no privilege on any auth table or function; every foreign key to `users` or `submissions` cascades.
- [ ] Email tokens are single-use (concurrent-safe), expire after exactly 15 minutes, are purpose-bound, superseded on re-issue, and stored only as hashes.
- [ ] OAuth uses `state` and PKCE S256, a single-use flow, a browser binding cookie for sign-in, and a session binding for linking.
- [ ] An OAuth email match never links, merges or signs in to another account (P5, P6 pass).
- [ ] Linking happens only through a signed-in user's own flow or a verification email opened by the same account.
- [ ] Sessions are opaque, hashed at rest, HttpOnly, Secure, SameSite=Lax, rolling 30 days with a 90-day cap, and renewed at most hourly; a new value is issued at every sign-in.
- [ ] `logout` and `logout-all` take effect on the next request; long-lived streams stop within one heartbeat.
- [ ] A session never reads another user's data; no handler takes a user id from the request.
- [ ] The Origin guard covers every non-safe `/api` route, including the run endpoints.
- [ ] Rate limits and the global email cap hold; mail endpoints fail closed when Valkey is down.
- [ ] Magic link, register and reset answers are identical for known and unknown addresses; password login failures are identical.
- [ ] The demo account signs in by password, is restricted by capabilities and by database triggers, is unreachable by magic link and OAuth, and resets daily without touching other users' data.
- [ ] No secret, token, session value, password, `code`, `state` or email address appears in any log line.
- [ ] No task read, wrote or created `.env` or any credential file.
- [ ] The 22 correctness properties each have a passing test; fmt, Clippy (`-D warnings`), workspace tests, web checks and `cargo deny check licenses` are green.
- [ ] The manual smoke checklist was run with real GitHub, Google and Resend and its results are recorded.
