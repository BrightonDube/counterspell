# Operations log

Started 16 November 2026. One line per event: date (UTC), type (deploy, rollback, rotation, drill, backup, incident, decision), reference (SHA, file name or task id), and outcome. Never write a secret value, a password, a token or a full connection string here (R11.7).

| Date (UTC) | Type | Reference | Outcome |
|---|---|---|---|
| 2026-11-16 | decision | deploy-ops spec | Repository-side deploy work implemented (T02 to T18 in part). Host work is held for T23 to T29, which Brighton runs. |
| 2026-11-20 | deploy | `13cd4fdb` | Signed server and judge started on the shared host after Valkey and hint configuration fixes. Judge preflight, all 19 migrations, internal readiness and public health passed. Break Room imported 3,000 Lichess puzzles. Starter-pack approval and the production Run and Submit check remain pending. |
| 2026-11-25 | deploy | batch-01 catalogue import | Ten batch-01 packs imported `created version=1` and published via `counterspell-catalogue` on the prod rootless daemon; `/api/problems` serves 15; one ops-verify submit run accepted 27/27. |
