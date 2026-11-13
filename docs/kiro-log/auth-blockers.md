# Auth blockers

Run: auth implement, attempt 1 (13 Nov 2026).

## T01: licence decision needed for reqwest and oauth2 (BLOCKED, Brighton)

- **Problem:** `reqwest 0.12` with the `rustls-tls` feature pulls in `webpki-roots 1.0.12`, which is licensed `CDLA-Permissive-2.0`. `cargo deny check licenses` rejects it, because `deny.toml` does not allow that licence. `oauth2 5.0` also depends on `reqwest`, so it fails the same way.
- **What I did:** added every other T01 crate (see `THIRD-PARTY-NOTICES.md`, "Auth dependencies"). The licence check passes without `reqwest` and `oauth2`. I did not edit `deny.toml`, because the spec says to ask first.
- **Unblock options (Brighton to pick):**
  1. Allow `CDLA-Permissive-2.0` in `deny.toml`. This is a data-only licence (root certificates), so the risk is low.
  2. Change the `reqwest` feature from `rustls-tls` to `rustls-tls-native-roots`. That drops `webpki-roots` and keeps the licence list unchanged, but it uses the operating system's certificate store.
- **Tasks affected until this is decided:** T14 (Resend sender), T18 (OAuth provider trait and PKCE via `oauth2`), T19 (GitHub), T20 (Google), T21 (OAuth sign-in), T22 (link provider), T23 (link email, depends on T14 and T17), T17 (magic-link handlers, depends on T14), T25 (register and reset, depends on T14), and T33 (wiring). Tasks that do not need these crates continue.
- **Assumed choice while blocked:** option 2 would be my choice, because it keeps the licence list unchanged. Not applied.

## T08: submissions foreign key deferred to T34 (ordering conflict)

- **Done:** the privilege matrix (`migrations/20261113110100_auth_privileges.sql`) and its tests (`crates/server/tests/pg_auth_privileges.rs`, 4 passing).
- **Not done:** the guarded `submissions.user_id → users(id) ON DELETE CASCADE` migration and its test "inserting a submission for an unknown user fails".
- **Why:** adding that key now makes every run-lifecycle test that creates a run for a random user id fail (`tests/enqueue.rs`, `tests/cancel.rs`, `tests/get_run.rs`). Those tests need real user fixtures first, which is T34's job. Moving the key into T08 would mean editing run-lifecycle's tests early.
- **Unblock:** do T33 and T34 (the fixtures), then add `migrations/<later timestamp>_auth_submissions_fk.sql` and the FK test. T08 can then be ticked. Assumed choice: keep the order as written in tasks.md and add the key in T34.
