# deploy-ops: done, pending tick

## Verified (spec-driver `verify all`, 17 Nov 2026)

| Task | Result |
|---|---|
| T07 | ticked. Judge `health::tests` and the heartbeat integration tests pass; `deploy/vps/compose.prod.yml` gives the `judge` service `healthcheck: test: ["CMD","counterspell-judge","healthcheck"]`; `bash deploy/vps/tests/compose_shape.sh` green; base `docker compose config` still works. |
| T08 | NOT ticked. Marked partial (access-log filter waits on auth T33). |
| T15 | NOT ticked. `compose_shape.sh` is green, but the task depends on auth T33 and judge-sandbox T38, which are both open. |
