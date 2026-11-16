# Monitoring and alerts

Every alert goes to Brighton's email address (R9.9). No paid pager, SMS or incident service is used. The two services below are free plans, and their terms are checked before setup (Q6). Brighton records the check and the date in `ACCOUNTS.md` (T01).

## Uptime: UptimeRobot (R9.7)

| Setting | Value |
|---|---|
| Monitor type | HTTPS keyword |
| URL | `https://<site>/api/health` (`<site>` is `CADDY_SITE_ADDRESS`) |
| Keyword | `"ok"` (the health route returns `{"status":"ok"}`) |
| Interval | 5 minutes |
| Alert contact | Brighton's email (Q7) |

Test: pause the site's Caddy container for two minutes (`docker compose ... stop caddy`), confirm the down email arrives, then start it again and confirm the up email arrives. Record both times in `OPERATIONS-LOG.md`.

## Dead-man checks: Healthchecks (R9.8)

A dead-man check alerts when a job stops pinging, so a silent failure becomes an alert. Each check has a name, a period and a grace time.

| Check name | Pinged by | Period | Grace | Ping URLs (kept in the host env file, never in git) |
|---|---|---|---|---|
| `counterspell-backup` | `counterspell-backup` (success on a complete run, fail on any error) | 1 day | 2 hours | `HC_SUCCESS_URL`, `HC_FAIL_URL` in `/etc/counterspell/backup.env` |
| `counterspell-ops-report` | `counterspell-ops-report` (success on a clean run; the fail URL on a threshold breach or a query error) | 5 minutes | 10 minutes | `HC_OPS_URL`, `HC_OPS_FAIL_URL` in `/etc/counterspell/ops.env` |
| `counterspell-certificate` | not yet in code (see below) | 1 day | 2 hours | to be set |
| `counterspell-restore-drill` | not yet in code (see below) | 35 days | 3 days | to be set |

Test each check the same way: stop its job's timer (`systemctl stop counterspell-backup.timer`), wait past the grace time, and confirm the missing-ping email. Then start the timer again. For the ops report, a threshold breach is tested by a fixture in `deploy/vps/tests/ops.bats`. Record each test in `OPERATIONS-LOG.md`.

### Not yet in code

- **Certificate expiry (R9.10).** No script checks the certificate yet. A daily check must read the expiry of the site certificate and ping the fail URL below 14 days. Until it exists, Brighton checks it by hand each month.
- **Restore drill.** `counterspell-restore-drill` writes the monthly record but does not ping Healthchecks. Until a ping is added, Brighton pings the drill check by hand after he writes the record.

These gaps are listed in `docs/kiro-log/deploy-ops-blockers.md`.

## Deploy result

The GitHub Actions run status is the deploy result. GitHub emails the account owner on a failed run (default GitHub notification settings).

## Plan terms (Q6)

Before the monitors are created, Brighton reads the UptimeRobot and Healthchecks free-plan terms, and records the plan name, the check limit and the date here. Neither service is paid. If the plan terms change to a paid tier, the monitors move or stop, and the decision goes into `OPERATIONS-LOG.md`.
