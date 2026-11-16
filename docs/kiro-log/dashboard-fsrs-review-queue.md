# dashboard-fsrs review queue

Items Brighton should check. Each is a SUPERVISED task or a change to a shared seam.

| Task | Commit | What Brighton should check |
|---|---|---|
| T06 privacy and aggregate tests | 58f7492 | `crates/server/tests/pg_dashboard_privacy.rs`. Removing the owner filter from `owned_completed` in `query.rs` makes `pg_each_learner_sees_only_their_own_progress_and_suggestions` fail (checked by a temporary mutation, then restored). Public concept names and problem slugs in suggestions are not private, so the canary checks read only the private sections. |
| T08 core checkpoint | 7c3210f | `fmt`, `clippy -D warnings`, 1026 workspace tests, 14 dashboard `pg_` and HTTP tests, `cargo deny check licenses`, `docker compose config`, `npm run check` (0 errors) and `npm run build` all passed. The server has no Docker or Valkey dependency on the dashboard path. T10 (browser smoke) is still open. |
