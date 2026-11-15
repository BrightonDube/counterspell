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

## Gaps found in review (fix before T24 and T28)

1. **Install list.** The app role installs only the two deploy scripts. The backup (`counterspell-backup` plus its service and timer), the ops report (script, SQL, service and timer) and the restore drill (script, service and the drill template) are not installed yet. Add them to `app_scripts` (and the unit files) before T28. The SQL files install to `/usr/local/share/counterspell/`, the drill template to `/usr/local/share/counterspell/restore-drill-template.md`.
2. **Host tools.** `jq` (ops report), `age` (backup and drill), `rclone` (backup and drill) and the `psql` client (ops report, role setup) are not installed by the base role. Add them to the base role's apt list before T24.
3. **Fifth and sixth env files.** `/etc/counterspell/ops.env` (cs_ops_ro login, see T17) and `/etc/counterspell/drill.env` (`DRILL_IDENTITY_FILE` and the bucket) are new. The app role's env check covers only the four service files. Brighton confirms both files and their modes (0600 for ops.env, 0600 for drill.env).
4. **Judge sandbox socket mount.** `compose.prod.yml` does not yet mount the sandbox socket into judge (the P1 red check). The T15 overlay adds it, as the design's judge block shows.
5. **Deploy-side `preflight`.** The deploy script refuses at the preflight step until `counterspell-judge preflight` exists (judge-sandbox T22 is in progress). This is deliberate.

## Status check, attempt 2 (16 Nov 2026)

- Every unchecked task that needs no host and no other open spec is done in code. T14 is ticked (red output re-run: 14 FAIL lines, exit 1, ShellCheck clean).
- T07: code and tests committed (5be8069). `cargo test -p counterspell-judge --test healthcheck` passes (3 tests); clippy is clean. The tick still waits for the `docker compose config` check in T15.
- T08: partial Caddy edge committed (2bfb6fe). The access-log query-string filter waits on auth T33, which is unchecked.
- T15 and T19 wait on auth T33 and judge-sandbox T38. Both are unchecked, so T15 was not started. T20 waits on judge-sandbox T38 (`renovate.json` exists, but T38 is not ticked).
- T01, T23 to T32 and T34 are `[HUMAN]` and stay unchecked.
- T33 (verification report) depends on T31 and T32, which are host work, so it stays unchecked.
- Unblock path: tick auth T33 and judge-sandbox T38, then run T08 (access log), T15 (Compose green), T07 tick, T19 and T20.
