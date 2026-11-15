# deploy-ops blockers

Items that need Brighton, another spec, or a decision Kiro may not take alone. Each entry gives the task id, the reason and what would unblock it.

## Brighton-only ([HUMAN]) tasks, left unchecked

| Task | Reason | Unblock |
|---|---|---|
| T01 | Creates accounts (registrar, OVH, object storage, UptimeRobot, Healthchecks) and records names. Spending hold (Review decisions). | Brighton lifts the spending hold for account creation, then records the names in `deploy/vps/ACCOUNTS.md`. |
| T23 | Orders the VPS (`ovh_vps` places a billable order). Needs T01 and the [U] catalogue values in `terraform.tfvars`. | Brighton reads the plan, runs apply on his machine, records IP and plan in `ACCOUNTS.md`. |
| T24 | Runs Ansible on the real host. | T23 done; Brighton runs the base role; confirm Docker version pins (`apt-cache madison`) first. |
| T25 | Runs docker, app and sandbox roles on the host; sandbox checks need a rootless daemon on the VPS. Depends on judge-sandbox T02 (open). | T24 done, and the judge `preflight` subcommand exists (see "Dependencies" below). |
| T26 | DNS, Resend domain verification and OAuth consoles. | T23, Q1 (domain), Q27 (callback URLs). |
| T27 | Writes `/etc/counterspell/*.env` by hand, creates Postgres roles, checks `cs_ops_ro`. Needs run-lifecycle deploy notes, T15 and T17. | T15, T17, T24, T26 done. |
| T28 | Off-box backup and first restore on the host and laptop. | T16, T18, T24, T27 done; bucket and `age` identity created by Brighton. |
| T29 | UptimeRobot and Healthchecks alert tests. | T22 done; accounts from T01. |

## Dependencies on other specs (tasks that cannot close yet)

| Task | Blocked by | Note |
|---|---|---|
| T07 tick | T15 | `docker compose config` must list the judge healthcheck. Code is committed; the tick waits for T15. |
| T08 (R6.5) | auth T33 (open) | Access-log query-string filter for the OAuth callback and verify routes. Also shares `Caddyfile` with auth T33; merge after it. |
| T08 (Q17) | Brighton decision | CSP is report-only. Enforcing needs a hash or nonce decision, because `web/build/200.html` has an inline bootstrap script. Brighton should choose: hash strategy at build time, or `'unsafe-inline'` for scripts. Until then Caddy sends `Content-Security-Policy-Report-Only`. |
| T14 | auth T33 and judge-sandbox T38 | Written before them, as the task allows. Not yet started. |
| T15 | auth T33 and judge-sandbox T38 | Both edit `docker-compose.yml`. Merge them first. |
| T20 | judge-sandbox T38 | Renovate file is created by T38. |
| T25 (sandbox) | judge-sandbox T02 (VPS re-run) | Host-only check. |
| T19 | T15 and the rest of Wave 2 | Depends on T14 through T18. |
| `counterspell-judge preflight` | judge-sandbox T22 (in progress; the `preflight` subcommand is not yet wired) | `counterspell-deploy` runs `run --rm judge preflight` before migrations. Until the subcommand exists, every deploy refuses at that step. This is deliberate (fail closed). |

## Decisions for Brighton

1. **Q17, CSP enforcement.** See above. Current state: report-only, strict directives, so violations are logged and the app is not broken.
2. **Judge heartbeat (Q26).** Chose a heartbeat file, written every 10 s by a task in `service::serve`, and read by `counterspell-judge healthcheck`. The path comes from `JUDGE_HEARTBEAT_FILE` (default `/tmp/counterspell-judge.heartbeat`). Confirm this is acceptable, or ask for a Postgres-based heartbeat instead (that would need a worker-side write per run and a new table).
3. **Docker version pins (T12).** `5:28.0.0-1~ubuntu.24.04~noble` and `2.24.0-1~ubuntu.24.04~noble` are placeholders. Confirm the real versions on the host (T24).
4. **Terraform and OVH.** `ovh_vps` is an ordering resource. Apply places a billable order, so it is held for T23. `prevent_destroy` is set. Confirm before T23 that the provider's plan codes and the subsidiary value match your account.
5. **Tooling installed on this machine (not repo dependencies).** bats 1.13.0, shellcheck 0.11.0 (`shellcheck-py`), terraform 1.17.2, tflint 0.66.0, ansible-core 2.22.0, ansible-lint 26.11.0, and the collections `community.general` 12.5.0 and `ansible.posix` 2.1.0 (galaxy). Checksums were checked for terraform and tflint; the others came from pip and npm.

## Notes on Kiro's own choices (already in the code)

- The deploy root script takes argv, not `SSH_ORIGINAL_COMMAND`. sudo clears that variable, so the entry point (`counterspell-deploy-entry`) passes fixed arguments. The bats harness was updated for this. No assertion was removed.
- The judge's clippy policy bans `Command::new`, so the healthcheck test calls `health::report` in-process rather than spawning the binary.
- `lint` on the Ansible roles uses the production profile. Role variables carry role prefixes (`app_`) to pass it.

## T17 decisions for Brighton (ops report)

1. **Kill-verdict counts are not measured.** R9.5 asks for `output_limit`, `time_limit` and `memory_limit`. In the real schema those are `submissions.verdict`, which R9.4 does not grant to `cs_ops_ro`. Granting `verdict` widens a security control, so it is not done. Options: (a) grant `verdict` to `cs_ops_ro` (the column holds no user content); (b) keep it out and measure those kills from the judge's own `run_events`. Brighton chooses. The report currently omits these counts and says so in `ops-report.sql`.
2. **A fifth env file, `/etc/counterspell/ops.env`.** The report needs the `cs_ops_ro` password. The design's list had four service files. This adds `ops.env` (root, 0600, written by hand at T27). Brighton confirms, or names another home for the password.
3. **Dead-job alert window.** R9.5 measures dead jobs over 24 hours, and R9.6 alerts on one in the last hour. The report prints both `dead_jobs_24h` and `dead_jobs_1h`, and alerts on the hour.
4. **P11 (grant check against Postgres 17): MODEL ONLY.** `ops-role.sql` and `ops-report.sql` were checked against the migration columns (`jobs`: state, run_after, lease_expires_at, updated_at; `submissions`: id, status, created_at, finished_at, verdict). They were not run on a server. The check is due when a Postgres 17 container is free (T27).
5. **`jq` and `psql` on the host.** The report needs `jq` and `psql`. The base role does not install them yet.
