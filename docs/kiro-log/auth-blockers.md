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

## Not started in attempt 1 (no blocker, out of time and context)

Tasks T10 to T37 are unchecked and were not started in this run. They are not blocked by a decision. Pick up at T10 (`SessionService`, SUPERVISED) in the next run. T14, T17, T18 to T23, T25 and T33 also need the T01 licence decision above. T27, T36 and T34 depend on those.

## Attempt 2 (13 Nov 2026, afternoon)

### T01 still blocked (licence decision)
Unchanged from attempt 1. Brighton has not chosen between allowing `CDLA-Permissive-2.0` in `deny.toml` and switching to `rustls-tls-native-roots`. Tasks that need `reqwest` or `oauth2` stay unchecked: T14, T17, T18, T19, T20, T21, T22, T23, T25, T27, T33, T34, T35, T36, T37.

### T08 FK still deferred to T34
Needs run-lifecycle fixtures (T34), which are blocked by the T01 list above.

### T28 daily demo reset: time source conflict (cut candidate)
`cs_reset_demo_account()` (migration `20261113110200_auth_demo.sql`) stamps `demo_reset_at` with the database clock. The reset schedule takes the application `Clock`, and P18 tests need a moved clock. Those two clocks disagree in tests, so "no reset before the hour" cannot be proven without changing the function. Options for Brighton:
1. Add a migration that passes the application time into `cs_reset_demo_account(p_now timestamptz)`. This changes a T09 contract.
2. Accept database time for the schedule and test only with real time.
Assumed choice: option 1 when T28 is picked up. T28 is on the cut list, so I left it unchecked.

### T29 partial: auth client exists, rune session store missing
`web/src/lib/auth/{client.js,messages.js,return-to.js}` and their Vitest tests (9 passing) are in place. `session.svelte.js` (the rune store) is not written, so T29 stays unchecked. T30, T31 and T32 are not started.

### Blocked by T01 and T34 (listed above) and by the Brighton review queue
Tasks T14 onward are unchecked; see the list above.
