# Deploy-ops verification report

Compiled from the task notes of T03 to T32. Statuses follow R12.7/R12.8: a host-level
guarantee stays `NOT TESTED` until a [HUMAN] task records evidence on the real host. No
VPS exists yet (T23 onward are Brighton's), so no `TESTED on host` line can appear here.

Legend: `TESTED in CI` = runs in GitHub Actions on every relevant change; `TESTED locally`
= verified on the development machine against real tooling; `MODEL ONLY` = the artefact
exists and is statically checked but its effect needs a host; `NOT TESTED` = not run.

## R1. Provider-neutral provisioning

- R1.1 module contract — `infra/terraform/modules/host/` exists with `variables.tf`/`outputs.tf`.
  `TESTED in CI` (terraform.yml: fmt -check, init -backend=false, validate, TFLint).
- R1.2 OVH under `providers/ovh/` only — `TESTED in CI` (same workflow lints the env).
- R1.3 version pinning — `versions.tf` exists and validates. `TESTED in CI`.
- R1.4 credentials from shell env only — `MODEL ONLY` (no credential variable has a default;
  verified by `terraform validate` requiring none).
- R1.5 no state/tfvars committed — `TESTED in CI` (deploy-checks.yml secret-names job).
- R1.6 CI runs fmt/validate/TFLint only — `TESTED in CI` (terraform.yml has no plan/apply step).
- R1.7 plan-order failure stops without substitution — `NOT TESTED` (needs OVH console).

## R2. Host configuration

- R2.1 idempotent playbook — `MODEL ONLY` (ansible-lint + --syntax-check in deploy-checks.yml;
  idempotence needs a host run).
- R2.2 three users — `MODEL ONLY` (roles exist under infra/ansible/roles/).
- R2.3 SSH hardening — `MODEL ONLY`.
- R2.4 firewall — `MODEL ONLY`.
- R2.5 unattended upgrades, no auto-reboot — `MODEL ONLY`.
- R2.6 pinned Docker Engine — `MODEL ONLY`.
- R2.7 timesyncd + preflight clock check — `MODEL ONLY` (preflight code exists; clock check is host-level).
- R2.8 no secrets through Ansible — `MODEL ONLY` (no secret-named variable in the playbooks;
  the secrets dir is populated by Brighton on the host).
- R2.9 forced-command authorized_keys line — `MODEL ONLY`.

## R3. Rootless sandbox daemon

- R3.1 systemd user service, separate socket — `MODEL ONLY` (unit files written; the local
  rootless daemon on the dev machine is the same shape and was exercised by the judge-sandbox
  T18/T39 runs — `TESTED locally`, not on the production host).
- R3.2 delegate.conf + controller check — `MODEL ONLY`.
- R3.3 user service caps (MemoryMax/TasksMax/CPUQuota) — `MODEL ONLY`.
- R3.4 POSIX ACL on the socket for the judge UID only — `MODEL ONLY` (locally the socket is
  used read-write by the judge client — `TESTED locally` as a mechanism, ACLs untested).
- R3.5 lingering — `MODEL ONLY`.
- R3.6/R3.7 image set + preflight probe — `TESTED locally` (preflight ran real language
  containers through the dev machine's rootless daemon during judge-sandbox verification).
- R3.8+ operational properties — `MODEL ONLY`/`NOT TESTED` on host.

## R4. Production Compose topology

- R4.1 images pinned by digest — `TESTED locally` (compose_shape.sh asserts all digests;
  will run in CI via deploy-checks.yml).
- R4.2/R4.3 networks and no published ports — `TESTED locally` (compose_shape.sh).
- R4.4 server/judge/migrate hardening — `TESTED locally` (read_only, cap_drop,
  no-new-privileges, pids_limit, mem_limit asserted by compose_shape.sh).
- R4.5 memory limits (postgres 1024m, valkey 160m, etc.) — `TESTED locally` (compose_shape.sh).
- R4.6 scram-sha-256, no POSTGRES_HOST_AUTH_METHOD — `TESTED locally` (compose_shape.sh).
- R4.7 server waits on migrate success — `TESTED locally` (compose_shape.sh).
- R4.8 json-file log cap 10 MB x5 — `TESTED locally` (compose_shape.sh x-logging anchor).
- R4.9 edge subnet + TRUSTED_PROXIES — `TESTED locally` (rendered config check; TRUSTED_PROXIES
  comes from host.env and is required, asserted by compose_shape.sh).
- R4.10 every base entry reading a host var is reset — `TESTED locally` (compose_shape.sh).

## R5. Secrets and configuration

- R5.1 per-service env files 0600 — `TESTED locally` for the wiring (env_file !override per
  service asserted); `MODEL ONLY` for the file modes on the host.
- R5.2 judge.env minimal — `TESTED locally` (compose_shape.sh asserts judge sees only its own env).
- R5.3 migrate/server env scopes — `TESTED locally` (same assertion).
- R5.4 host.env copied into the release record — `MODEL ONLY` (deploy script does it; no host yet).
- R5.5 secrets out of git — `TESTED in CI` (secret-names job + trufflehog in security.yml).
- R5.6 secrets created by hand — `NOT TESTED` (Brighton's task T27).
- R5.7 per-service Postgres logins via NOLOGIN roles — `MODEL ONLY` (roles exist in
  migrations; login wiring on the host untested).
- R5.8 cs_ops_ro column-level SELECT — `MODEL ONLY` (initdb SQL exists).

## R6. Edge, TLS and HTTP headers

- R6.1 site address env with :80 default — `TESTED locally` (caddy validate passes for both
  `:80` and a real hostname; caddy_shape.sh).
- R6.2 HTTP->HTTPS redirect — `MODEL ONLY` (Caddy's automatic behaviour once a hostname is
  set; not exercised).
- R6.3 headers + Server removal — `TESTED locally` (caddy_shape.sh asserts every header).
- R6.4 SSE unbuffered — `TESTED locally` (flush_interval -1 asserted by text and validate).
- R6.5 query-string filter on OAuth callback/verify routes — `TESTED locally`
  (log filter replaces `code`, `state`, `token` with REDACTED; validated in the pinned image).
- R6.6 /api/ready answers 404 — `TESTED locally` (respond rule before the proxy; both asserted).

## R7. Release and rollback

- R7.1 signed+attested digests only — `MODEL ONLY` (deploy.yml signs; deploy script verifies).
- R7.2 pin by digest — `TESTED locally` (counterspell-deploy.bats asserts digest parsing).
- R7.3 migrate before start — `TESTED locally` (bats suite asserts ordering with stub docker).
- R7.4 failed migration leaves release — `TESTED locally` (bats stubs).
- R7.5 preflight before judge — `TESTED locally` (counterspell-deploy calls preflight; the
  subcommand was exercised against the dev rootless daemon).
- R7.6 readiness inside the stack + public health — `TESTED locally` (bats stubs).
- R7.7 auto-restore on failed checks — `TESTED locally` (bats stubs).
- R7.8 previous release record — `TESTED locally` (bats stubs).
- R7.9 rollback swaps records, no migrate — `TESTED locally` (bats stubs).
- R7.10 expand-then-contract migrations — `MODEL ONLY` (convention; not mechanically checked).
- R7.11 exclusive lock — `TESTED locally` (flock stub test).
- R7.12 sudo rule, no args wildcard, no docker group — `MODEL ONLY` (sudoers fragment exists).
- R7.13 compose checksum refusal — `TESTED locally` (compose_sha.bats).
- R7.14 status prints no secret — `TESTED locally` (bats asserts output shape).
- R7.15 `production` environment gate — `MODEL ONLY` (workflow environment key set; the
  environment itself is created in the repo settings by Brighton).
- R7.16 RELEASE.md checklist — `TESTED locally` (file exists).

## R8. Backups and restore

- R8.1 nightly pg_dump timer 01:30 UTC — `MODEL ONLY` (timer unit written).
- R8.2 age encryption to public key — `TESTED locally` (backup.bats with a throwaway age key).
- R8.3 keep 7 newest — `TESTED locally` (backup.bats).
- R8.4 off-box copy — `TESTED locally` (rclone stub) / `NOT TESTED` against real storage.
- R8.5 private key off the server — `MODEL ONLY` (key material is never provisioned).
- R8.6 30-day lifecycle — `MODEL ONLY`.
- R8.7 failure keeps dumps + signals dead-man — `TESTED locally` (backup.bats failure leg).
- R8.8 monthly drill — `TESTED locally` (drill.bats with fake pg_dump/age/rclone); real drill
  `NOT TESTED` (T28).
- R8.9 failed drill recorded — `TESTED locally`.
- R8.10 drill output has counts/hashes only — `TESTED locally` (drill.bats asserts no data).

## R9. Observability and alerts

- R9.1 LOG_FORMAT=json — `MODEL ONLY` (env-file fixtures set it).
- R9.2 no user code/secrets in logs — `MODEL ONLY` (canary test is T31 on the host;
  `NOT TESTED` here).
- R9.3 journald cap 500 MB — `MODEL ONLY`.
- R9.4 ops report via cs_ops_ro column SELECT — `TESTED locally` (ops.bats with a fake psql).
- R9.5 ops report measurements — `TESTED locally` (ops.bats asserts the query columns).
- R9.6 failure signalling — `TESTED locally` (ops.bats).
- R9.7 external uptime monitor — `NOT TESTED` (T29, Brighton's monitor account).
- R9.8 dead-man checks — `TESTED locally` (stub) / `NOT TESTED` against the real monitor.
- R9.9 alerts by email only — `MODEL ONLY`.
- R9.10 certificate expiry check — `MODEL ONLY` (timer written).

## R10. Health and readiness routes

- R10.1 /api/health + /api/ready 200/503 — `TESTED in CI` (crates/server health tests; ready
  returns 503 when a dependency times out, body has no connection detail).
- R10.2 counterspell-judge healthcheck exits 0/1 on heartbeat freshness — `TESTED in CI`
  (judge crate tests) and used in compose.prod.yml.
- R10.3 compose uses judge healthcheck; deploy uses /api/ready — `TESTED locally`
  (compose shape + deploy bats).

## R11. Security operations

- R11.1 deploy-key rotation procedure — `MODEL ONLY` (OPERATIONS-LOG/RUNBOOK entries).
- R11.2 digest updates as reviewed PRs, never auto-merged — `TESTED locally`
  (renovate.json regex manager matches the compose digests; automerge is false repo-wide).
- R11.3 sandbox-escape runbook — `MODEL ONLY` (deploy/vps/RUNBOOK.md).
- R11.4 encrypted evidence archive with SHA-256 manifest — `MODEL ONLY`.
- R11.5 availability window until 29 Jan 2027 — `NOT TESTED` (runtime guarantee).
- R11.6 post-freeze change policy — `MODEL ONLY` (policy recorded).
- R11.7 OPERATIONS-LOG.md for every event — `MODEL ONLY` (file exists; use starts with the
  first deploy).

## R12. Verification without a server

- R12.1 Terraform fmt/validate/TFLint in CI — `TESTED in CI` (terraform.yml).
- R12.2 ansible-lint + --syntax-check — `TESTED in CI` (deploy-checks.yml ansible job);
  idempotence in a container is `NOT TESTED`.
- R12.3 ShellCheck + bats with stubs — `TESTED in CI` (deploy-checks.yml shellcheck and bats
  jobs); every suite under deploy/vps/tests ran locally with stubs.
- R12.4 docker compose config + shape test — `TESTED locally` and wired into
  deploy-checks.yml.
- R12.5 caddy validate in the pinned image — `TESTED locally` (caddy_shape.sh) and wired
  into deploy-checks.yml.
- R12.6 backup/ops/drill with fakes — `TESTED locally` (bats stubs; age keys are tempdir-only).
- R12.7 host checks listed as [HUMAN] — `TESTED` (T23-T32 carry the commands and result files).
- R12.8 no host guarantee marked tested without evidence — `TESTED` (this report marks every
  host-level item MODEL ONLY or NOT TESTED).
