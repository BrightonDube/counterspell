# Auth review queue

Each item is a `[SUPERVISED]` task. Brighton reviews the diff before the next dependent task starts.

- **T05 (token primitives)**: `crates/server/src/auth/tokens.rs`. Check that `RawToken::generate` returns `Result` (a deliberate change from the design's `-> Self`, so no `expect` is needed outside tests), that the type has no `Clone` and a redacted `Debug`, and that `token_status` matches the expiry rule. Commit hash: see the T05 commit in `git log`.
