# Auth review queue

Each item is a `[SUPERVISED]` task. Brighton reviews the diff before the next dependent task starts.

- **T05 (token primitives)**: `crates/server/src/auth/tokens.rs`. Check that `RawToken::generate` returns `Result` (a deliberate change from the design's `-> Self`, so no `expect` is needed outside tests), that the type has no `Clone` and a redacted `Debug`, and that `token_status` matches the expiry rule. Commit hash: see the T05 commit in `git log`.
- **T06 (pure decisions and capabilities)**: `crates/server/src/auth/decision.rs`, `crates/server/src/auth/capabilities.rs`. Check the order of checks in `decide_link`, that `decide_oauth_login` has no path to `SignIn` for a matching email, and the one `#[allow(clippy::struct_excessive_bools)]` on `Capabilities` (five flags are fixed by the UI contract).
- **T07 (core auth tables, migration `20261113110000_auth_core.sql`)**: check the `sessions` trigger that blocks owner, creation and absolute-limit changes, and the `oauth_flows` CHECK that separates login (binding, no session) from link (session, no binding). `sqlx::test` embeds migrations at compile time, so a new migration needs a rebuild of the test crate (touch the test file).
