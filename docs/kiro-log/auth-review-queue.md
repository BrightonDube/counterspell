# Auth review queue

Each item is a `[SUPERVISED]` task. Brighton reviews the diff before the next dependent task starts.

- **T05 (token primitives)**: `crates/server/src/auth/tokens.rs`. Check that `RawToken::generate` returns `Result` (a deliberate change from the design's `-> Self`, so no `expect` is needed outside tests), that the type has no `Clone` and a redacted `Debug`, and that `token_status` matches the expiry rule. Commit hash: see the T05 commit in `git log`.
- **T06 (pure decisions and capabilities)**: `crates/server/src/auth/decision.rs`, `crates/server/src/auth/capabilities.rs`. Check the order of checks in `decide_link`, that `decide_oauth_login` has no path to `SignIn` for a matching email, and the one `#[allow(clippy::struct_excessive_bools)]` on `Capabilities` (five flags are fixed by the UI contract).
- **T07 (core auth tables, migration `20261113110000_auth_core.sql`)**: check the `sessions` trigger that blocks owner, creation and absolute-limit changes, and the `oauth_flows` CHECK that separates login (binding, no session) from link (session, no binding). `sqlx::test` embeds migrations at compile time, so a new migration needs a rebuild of the test crate (touch the test file).
- **T09 (demo guards, seed, reset; migration `20261113110200_auth_demo.sql`)**: check `cs_guard_demo_user` (no insert, delete, credential change or demo-flag change outside the seed), `cs_guard_demo_identity` (no identity for the demo), `cs_seed_demo_user` (the setting is local to the transaction), and `cs_reset_demo_account` (only finished submissions and the demo user's rows, advisory lock, fixed `search_path`).

Attempt 2 supervised tasks (each commit can be reviewed with `git show <hash>`):
- **T10 (sessions)**, commit `19d5dd1`: check the SQL in `session.rs` against design §Sessions (renewal guard `last_renewed_at <= now - 1h`, `LEAST` cap) and that `authenticate` never returns a user for an expired row.
- **T11 (cookies, extractors, logout, /api/me)**, commit `1e50811`: check that `UserResolver` became `async_trait` (a deliberate seam change, see its commit body), that no handler reads a user id from input, and that the cookie slot is the only way renewed cookies are sent.
- **T12 (session property test)**, commit `8f34841`: the renewal-throttle mutation was run and caught (failing seed committed in `pg_auth_session_props.proptest-regressions`). Reverting `RENEW_EVERY` to 2 h makes the property fail.
- **T13 (TokenStore)**, commit `c6aea37`: check that `consume` is one `UPDATE ... RETURNING` and that `issue` supersedes in the same transaction.
- **T24 (password login)**, commit `889df01`: check the read-first limit (`AuthLimiter::check` reads the counter, `hit` counts only failures), the uniform `401`, and that `HashGate` wraps every verify and hash. Note: `ratelimit.rs` (run-lifecycle) gained an additive `current()` read.
- **T26 (demo seed and config)**, commit `eaa2a3d`: check that the seed goes only through `cs_seed_demo_user` and that `GET /api/auth/config` carries no secret.

Attempt 3 supervised tasks:
- **T28 (daily demo reset)**, commit recorded in `git log --grep "auth T28"`: a new migration `migrations/20261113110300_auth_demo_reset.sql` replaces the zero-argument `cs_reset_demo_account()` with `(p_now, p_due)`, so the schedule uses the application clock (the assumed choice from `auth-blockers.md`). Check: (1) the advisory lock is taken before `demo_reset_at` is read, so a second caller sees the first caller's reset; (2) the generic loop now excludes `sessions`, `identities`, `email_tokens`, `oauth_flows` and `submissions` by OID. The earlier name-based exclusion never matched, so the T09 version deleted the demo's sessions on reset. The new test `pg_reset_keeps_running_work_other_users_and_demo_sessions` caught it. (3) `pg_auth_demo_db.rs` names the new signature, with `test-change: R18.8`.

Attempt 4 supervised items (T14 is not supervised, but it has a deliberate exception to review):

- **T14 (mail senders, partial), `crates/server/src/auth/email.rs`**: `LogSender` writes the full sign-in text, including the `#token=` link, to the log under the target `counterspell::dev_mail`. This is a deliberate development-only exception, because design §Email sending and T33 need the link in the dev log. Check that: (1) `LogSender::new` refuses `AppEnv::Production` (it does, and a test covers it); (2) `GuardedSender` is the only sender wired in production (T33). Decide whether you accept this exception before T33 is ticked. T35's log-capture test should use `MemorySender`, not `LogSender`.

## T17 magic-link request and verify (SUPERVISED)
- Commit: see `git log --oneline --grep "auth T17"`.
- Check: `handlers/magic.rs` answers `202` before any account lookup, and the demo address gets no token and no mail. `handlers/verify.rs` consumes the token in one statement, and creates the account through `INSERT ... ON CONFLICT DO NOTHING`, refusing the demo row. Only the `MagicLink` purpose can sign in; the other purposes are refused until T23 and T25.
- Design choice: `AuthState` gained `clock`, `tokens` and an optional `mail` (`MailPort`), set by `with_mail`. `AuthState::new` keeps its signature, so existing tests still compile. T33 must call `with_mail` in production.
- Tests: `crates/server/tests/auth_magic.rs` (8 `pg_` tests, all passing against the dev Postgres).

## T18 provider trait and flow store (SUPERVISED)
- Check: `auth/oauth/flow.rs`. `consume` deletes the row before checking provider, expiry or binding, so every callback uses up its flow. `binding_matches` compares hashes in constant time.
- Deviation from the task list: `FakeProvider` is not in this commit. It is first used by T21's callback tests, so it is added there. The task text names it, so this is a known gap until T21.
- TDD note: the 7 `pg_auth_flow` tests were written first, but I did not run them red before writing the flow store. They pass now (`cargo test -p counterspell-server --test pg_auth_flow -- --include-ignored`).

## T21 OAuth sign-in start and callback (SUPERVISED)
- Check `handlers/oauth.rs`: `finish` runs in this order: take the flow, check the binding cookie (login only), check `error`, then exchange the code. The binding cookie is cleared on every outcome. Success and failure are redirects only.
- Check the no-linking rule in `sign_in`: the only email lookup is `profile_email`, which marks an address as taken when any account holds it, including the demo account. An existing identity is the only way to select an account.
- `CookieSlot` now holds several headers: `set` replaces, `push` adds. Existing callers are unchanged.
- `AuthState` gained `flows` and an optional `oauth` port (`with_oauth`). `AuthState::new` keeps its signature.
- Deviation: P6 asks for a randomised model test. I wrote a fixed four-step sequence (`pg_auth_oauth_model_a_sequence_of_callbacks_keeps_one_account_per_identity`). A proptest over random profile sequences is not written yet.
- Deviation: the link branch returns `oauth_failed` until T22 implements it.
- TDD note: as in T18, the tests were written before the handler but not run red first.
- Tests: `crates/server/tests/pg_auth_oauth_model.rs` (10 pg tests). The fake provider is `auth/oauth/fake.rs`, behind the existing `testkit` feature.

## T22: link a provider identity (commit aa7188e)
- Check `handlers/oauth.rs` `link_provider` and `flow_session_user`: the session must be the very session the flow was created from, and it is checked after the flow is taken.
- Check `handlers/link.rs`: the demo check runs before any flow is created (test: no `oauth_flows` row for the demo account).
- Known behaviour: when logout deletes the session, the cascade also deletes the link flow. The callback then cannot tell it was a link flow, so its error goes to `/login?error=state_invalid`, not to settings. The spec only requires that the flow stops working.

## T23: link an email address (commit: see git log "auth T23")
- Check `handlers/link.rs` `email` and `address_is_free`: the answer is `202` whether or not the address is taken, and no mail goes to a taken or demo address.
- Check `handlers/verify.rs` `link_email`: a session is required, a token bound to another account is refused without being used up, and the `UPDATE` is guarded by `email IS NULL`.
- Known behaviour: the token is used before the `UPDATE`. If another account takes the address in between, the answer is `409 email_in_use` and the token is spent. The user must ask for a new link, which is then refused because the address is taken.

## T25: register and password reset (Should; commit: see git log "auth T25")
- Check `handlers/register.rs`: the password is hashed before the link is issued, and no account row is written until the link is opened (`finish_registration`).
- Check `handlers/reset.rs` `confirm_reset`: the policy runs before the token is used, every session is revoked, and nobody is signed in.
- Known behaviour: a link for an address that already has an account is still mailed, and opening it gives `account_exists` and spends the link. This matches R15.7 and keeps the request answer uniform.
- `is_unique_violation` moved to `errors.rs`, and the copies in `oauth.rs` and `verify.rs` now use it.

## T27: capabilities everywhere (commit: see git log "auth T27")
- No production change was needed. T22 to T25 already call `Capabilities::require` in each mutating handler. This task adds the matrix tests in `tests/pg_auth_demo_matrix.rs`.
- Check `MUTATING_ROUTES` in `auth/mod.rs`: a route added to the router must also be added there. The test catches a listed route that the router does not serve, but it cannot find a served route that is not listed.
- The DELETE account route (T36) does not exist yet. When T36 lands, add its capability check and add `/api/me` (DELETE) to the matrix.
