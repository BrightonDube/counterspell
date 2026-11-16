# Run lifecycle: definition-of-done results

Recorded from one run of `pg_dod_twenty_concurrent_runs` (T36). The tables below are the values in `target/run-lifecycle/dod-report.json` from that run. A check confirmed that every table value appears in the file. Client and database metrics are in separate tables, as the design requires.

## Run

- Date: 15 November 2026, 20:43 SAST (UTC+2).
- Machine: WSL 2 on Windows, kernel `6.18.33.2-microsoft-standard-WSL2`, AMD Ryzen 5 7600 6-Core (12 logical CPUs), 7.8 GiB RAM, Docker Desktop and a rootless Docker daemon both running.
- Database: the Postgres 17 dev container (`cs-dev-pg`) on 127.0.0.1. Each test creates and migrates its own fresh database.
- Shape: 20 concurrent runs from 20 users, 3 judge workers with concurrency 2 each, scripted executor (compile, three hidden tests of 40 ms each). Five clients dropped their stream after two events and resumed with `Last-Event-ID`. Five other users and one anonymous client probed every run's events, read and cancel endpoints during the run.
- Result: all assertions passed (no lost jobs, one `done` per run, gap-free `seq`, five events delivered per stream, probes answered only 401 or 404, streams equal the stored log, no canary, enqueue p95 within 300 ms, every run terminal within 60 s).

## Client-side latency (client monotonic clock)

| Metric | Count | Mean ms | p50 ms | p95 ms | Max ms |
|---|---:|---:|---:|---:|---:|
| Enqueue (client, POST to 202) | 20 | 153.45829815 | 153.379663 | 168.003669 | 168.862342 |
| First event (client's own 202 to its first SSE event) | 20 | 1227.1266444500002 | 1211.8753410000002 | 1477.815634 | 1499.0240569999999 |
| Completion (client's POST to terminal event) | 20 | 1531.4929076499998 | 1512.9256010000001 | 1792.4455429999998 | 1814.3803229999999 |

## Database-side timings (database clock) and delivery

| Metric | Count | Mean ms | p50 ms | p95 ms | Max ms |
|---|---:|---:|---:|---:|---:|
| Queue wait (started_at minus created_at) | 20 | 1193.7687500000002 | 1207.034 | 1474.199 | 1495.442 |
| Execution (finished_at minus started_at) | 20 | 182.92749999999995 | 149.351 | 307.383 | 320.502 |
| Delivery (terminal event received minus its written time) | 20 | 4.5724365234375 | 4.531982421875 | 5.6796875 | 5.904296875 |

Delivery compares the client's wall clock with the database's `created_at` on the same host, so it is an estimate. Clock differences of a few ms are possible.

## What the numbers show

- The enqueue p95 is 168.0 ms, under the 300 ms target (R2.6).
- Queue wait averages 1194 ms, which is close to the judge's default claim poll of 1000 ms (`CLAIM_POLL_MS`). Judges learn about new work only when they poll, because the `NOTIFY run_jobs` wake-up is not wired yet (noted in T17). Waking claims on that notification would shorten this wait. Brighton to decide before the freeze.
- Execution is short here because the scripted executor does no real work. It says nothing about sandbox runtime.

## Not covered by this measurement

- Real sandbox execution: the executor is scripted. The judge-sandbox spec measures container start-up and run time.
- Real process crashes: "judge crash" is modelled by aborting a task, not by killing a process. The restart tests cover recovery through the database.
- Hidden-data canary: the DoD scan uses a marker that the scripted judges never hold. The canary test T34 covers a judge that holds hidden data. `compile_output` is not scanned for hidden markers (see the review queue).
- Caddy and browser streaming: the SSE path through Caddy and the browser is not tested (E2E, deferred).
- GitHub Actions: the CI workflow changes (T37) were run locally with the same commands, but not on GitHub.
- Variance: this is one run. The design asks for one recorded run, so no spread is reported.
