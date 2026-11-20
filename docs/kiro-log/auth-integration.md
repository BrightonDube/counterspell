# Auth and run-lifecycle integration (auth T34)

Status: code and tests done in the `kiro/drive-queue` run. **T34 is not ticked**, because its dependency T33 (server wiring, Compose and the Caddy log filter) is still open.

## What changed
- `UserResolver` (`crates/server/src/seams.rs`) gained a default method `still_signed_in(headers, user) -> bool`. The default asks `resolve` again, so every existing resolver (test resolvers included) is unchanged.
- `SessionUserResolver` overrides it with the read-only `SessionService::is_valid` on the hashed cookie. It never renews the session, so an open stream cannot keep a session alive. A database failure counts as signed out (fail closed): the stream closes and the client reconnects, and is refused with `401` if the session is really gone.
- The run event stream (`crates/server/src/runs/sse.rs`) keeps the opening request's headers and re-checks once per heartbeat interval (`SSE_HEARTBEAT_SECS`, default 15 s). It checks before sending any queued event, so nothing is sent after a sign-out.

## Evidence (R11.6, R12.2, R12.5, R18.7)
`crates/server/tests/pg_auth_runs_integration.rs`, 7 tests, all run against a real database and the full router:
- a session cookie authenticates `POST /api/runs`;
- a second user gets `404` for the first user's run;
- after logout-all the next request is refused and other users are untouched;
- the production resolver reads the session, not a fixed identity;
- **new:** logout-all closes an open event stream within the next heartbeat, and a new stream with the dead session gets `401`;
- **new:** a valid session keeps its stream open across several heartbeats (control for the test above);
- **new:** `still_signed_in` leaves `last_renewed_at` unchanged two hours later, and refuses a forged cookie and a missing cookie.

The close test failed before the change (the stream stayed open after logout-all) and passes after it.

## Limits
- A stream notices a sign-out at the next heartbeat check, so up to one heartbeat interval later (15 s by default), not instantly.
- The check costs one primary-key lookup per open stream per heartbeat.
- The stream is closed on any session failure, including a database error. That is deliberate.
