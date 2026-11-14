# Run lifecycle: deployment notes

This note covers the server and judge settings for the run lifecycle (submissions, the judge queue and SSE). It names variables only. Real values come from the host environment or a secret store, and are never committed.

## Database login roles

Migrations create two `NOLOGIN` roles with least privilege: `counterspell_server` and `counterspell_judge`. Give each service its own login:

1. Connect as the owner (`counterspell`) and create a login per role with a password typed in the shell. Never write the password to a file.
2. Grant the role membership: `GRANT counterspell_server TO <server_login>;` and `GRANT counterspell_judge TO <judge_login>;`.
3. Set `SERVER_DATABASE_URL` for the server and `JUDGE_DATABASE_URL` for the judge. Compose reads both from the host environment. Each URL uses its own login.

The judge must not use the owner role. The server must not be able to write `run_events`, `test_results` or verdicts (the role tests in `crates/judge/tests/roles.rs` check this).

## Configuration

| Variable | Used by | Purpose |
|---|---|---|
| `DATABASE_URL` | server, judge | Postgres login URL. Compose sets it from `SERVER_DATABASE_URL` or `JUDGE_DATABASE_URL`. |
| `VALKEY_URL` | server | Rate-limit counters only. Valkey is never a queue or event bus. |
| `HOST`, `PORT`, `STATIC_DIR` | server | Listen address and static web files. |
| `TRUSTED_PROXIES` | server | Proxies whose `X-Forwarded-For` is trusted for per-IP limits. |
| `REQUEST_BODY_LIMIT_BYTES`, `MAX_SOURCE_BYTES` | server | Request and source size limits (VAL-1). |
| `RUN_RATE_USER`, `RUN_RATE_IP`, `RUN_ACTIVE_CAP_PER_USER`, `RUN_QUEUE_DEPTH_MAX` | server | Run/Submit rate limits and queue caps. |
| `JUDGE_CONCURRENCY` | judge | Runs executed at once. Compose sets 2. |
| `LEASE_SECS`, `RENEW_SECS` | judge | Lease length and renewal interval. Compose sets 30 and 10. |
| `CLAIM_POLL_MS`, `SHUTDOWN_GRACE_SECS` | judge | Claim polling and graceful shutdown. |
| `MAX_CLAIM_AGE_SECS`, `QUEUE_TTL_SECS`, `REAPER_INTERVAL_SECS` | judge | Reaper limits for stuck runs and stale queued jobs. |
| `RETRY_BACKOFF_BASE_SECS`, `RETRY_BACKOFF_CAP_SECS` | judge | Retry backoff after an infrastructure failure. |

Defaults and validation rules are in `crates/server/src/config.rs` and `crates/judge/src/config.rs`. Invalid values stop the service at start-up with a clear error.

## Single server process

The SSE replay and run-event wake-ups assume one server process per database. Wake-ups are hints only: each client still replays from Postgres after its `Last-Event-ID`. Running several server processes is safe for replay, but per-user stream limits are counted per process, so a user could hold more streams than the limit. Keep one server process until that limit is moved into Postgres or Valkey counters.

## Caddy and event streams

The `Caddyfile` routes `GET /api/runs/{id}/events` to the server with `flush_interval -1` and without compression, so each event reaches the browser as it is written. All other routes keep `zstd` and `gzip` compression. Check the route with `caddy validate` and an SSE request after deployment.

## Judge isolation

The judge has no published ports and does not mount the Docker socket. Sandbox containers run on the separate rootless daemon described in `docs/system-design.md` §7. The server never reaches any Docker daemon.

## Reading the DoD report

The 20-run load test (`crates/server/tests/dod_twenty_runs.rs`, task T36) writes `target/run-lifecycle/dod-report.json`. Enqueue latency and completion latency are reported in separate tables. Copy the numbers from that file into `docs/kiro-log/run-lifecycle-dod.md` rather than typing them by hand. Until T36 has run, there is no report to read.
