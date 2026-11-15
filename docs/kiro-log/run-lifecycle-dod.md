# Run lifecycle: definition-of-done results

Recorded from the latest run of `pg_dod_twenty_concurrent_runs` (T36). Every number below is copied from `target/run-lifecycle/dod-report.json` by a script, not typed by hand. Client and database metrics are in separate tables, as the design requires.

## Run

- Date: 15 November 2026, 20:17 SAST (UTC+2).
- Machine: WSL 2 on Windows, kernel `6.18.33.2-microsoft-standard-WSL2`, AMD Ryzen 5 7600 6-Core (12 logical CPUs), 7.8 GiB RAM, Docker Desktop and a rootless Docker daemon both running.
- Database: the Postgres 17 dev container (`cs-dev-pg`) on 127.0.0.1. Each test creates and migrates its own fresh database.
- Shape: 20 concurrent runs from 20 users, 3 judge workers with concurrency 2 each, scripted executor (compile, three hidden tests of 40 ms each). Five clients dropped their stream after two events and resumed with `Last-Event-ID`. Five other users and one anonymous client probed every run's events, read and cancel endpoints during the run.
- Result: all assertions passed (no lost jobs, one `done` per run, gap-free `seq`, probes answered only 401 or 404, streams equal the stored log, no canary, enqueue p95 within 300 ms, every run terminal within 60 s).

## Client-side latency (client monotonic clock)

| Metric | Count | Mean ms | p50 ms | p95 ms | Max ms |
|---|---:|---:|---:|---:|---:|
| Enqueue (client, POST to 202) | 20 | 134.54739820000003 | 134.605266 | 148.332343 | 149.13986100000002 |
| First event (202 to first SSE event) | 20 | 1232.6890212000003 | 1251.629668 | 1478.206783 | 1522.593354 |
| Completion (POST to terminal event) | 20 | 1544.1179593000002 | 1557.064132 | 1773.035465 | 1821.916206 |

## Database-side timings (database clock) and delivery

| Metric | Count | Mean ms | p50 ms | p95 ms | Max ms |
|---|---:|---:|---:|---:|---:|
| Queue wait (started_at minus created_at) | 20 | 1212.6912 | 1232.684 | 1475.344 | 1517.23 |
| Execution (finished_at minus started_at) | 20 | 194.01944999999998 | 178.885 | 276.383 | 284.293 |
| Delivery (terminal event received minus its written time) | 20 | 5.0501953125 | 4.87744140625 | 6.345458984375 | 6.69140625 |

Delivery compares the client's wall clock with the database's `created_at` on the same host, so it is an estimate. Clock differences of a few ms are possible.

## What the numbers show

- The enqueue p95 is 148.3 ms, under the 300 ms target (R2.6).
- Queue wait averages 1213 ms, which is close to the judge's default claim poll of 1000 ms (`CLAIM_POLL_MS`). Judges learn about new work only when they poll, because the `NOTIFY run_jobs` wake-up is not wired yet (noted in T17). Waking claims on that notification would shorten this wait. Brighton to decide before the freeze.
- Execution is short here because the scripted executor does no real work. It says nothing about sandbox runtime.

## Not covered by this measurement

- Real sandbox execution: the executor is scripted. The judge-sandbox spec measures container start-up and run time.
- Real process crashes: "judge crash" is modelled by aborting a task, not by killing a process. The restart tests cover recovery through the database.
- Hidden-data canary: the DoD scan uses a marker that the scripted judges never hold. The canary test T34 covers a judge that holds hidden data. `compile_output` is not scanned for hidden markers (see the review queue).
- Caddy and browser streaming: the SSE path through Caddy and the browser is not tested (E2E, deferred).
- GitHub Actions: the CI workflow changes (T37) were run locally with the same commands, but not on GitHub.
- Variance: this is one run. The design asks for one recorded run, so no spread is reported.
