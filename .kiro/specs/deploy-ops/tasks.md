# Tasks: deploy-ops

Approved for implementation by Brighton on 15 Nov 2026 10:26, for code, IaC and docs only (`.kiro/steering/brighton-decisions.md`, and Review decisions in `design.md`). No purchases, no provisioning, no real servers, and no spending. T01 and other `[HUMAN]` account-creation tasks stay blocked.

Conventions:

- `[P]` can run in parallel with other tasks of its wave once its dependencies are done. `[S]` is sequential.
- `[SUPERVISED]` means Brighton reviews the change before it is accepted (deploy, sandbox or secret handling).
- `[HUMAN]` means a person must act on the VPS, a provider console, or a laptop that Kiro cannot reach. Kiro writes the exact commands and the result file. Brighton records the result.
- `[GATE]` means nothing downstream starts until the gate is recorded.
- Each task is sized for 30 minutes or less. Each Rust task ends with a short **Rust lesson**.
- Shell tasks use ShellCheck and bats with stub binaries. No test calls a real service, a real registry, a real provider API or a real host.
- Secrets: no task writes a secret value into the repository, a log, a spec, a file in `docs/` or a chat message. The only secret-bearing files are the host env files, which Brighton writes by hand (T27).
- Dependencies on other specs use their task ids. `run-lifecycle` T17 (judge heartbeat), T19 (judge wiring), T20 (server library and state) are done. `auth` T33 and `judge-sandbox` T02, T22, T37, T38 are open and are named where a task depends on them.

## Wave 0: gate and decisions

- [x] **T00 [S] [GATE] Spec approval (Brighton)**
  - Read `requirements.md` and `design.md`. Check the gap analysis (G1 to G22) and the deviations (D1 to D10). Answer Q1 to Q27 (each lists the assumed choice).
  - Output: a "Review decisions" section at the end of `design.md` with one line per answer. Any answer that changes a control (sudo rule, socket ACL, network split, env file scope, egress) updates `requirements.md` first.
  - Depends on: none.
  - Verify: no task below starts before this section exists.

- [ ] **T01 [S] [HUMAN] (20 min) Create the accounts and record their names**
  - Files: `deploy/vps/ACCOUNTS.md` (names, URLs and plan names only; no keys, passwords or tokens).
  - Brighton opens or confirms: the domain registrar account (Q1), the OVH account (Q2), the object-storage account (Q4), the UptimeRobot account (Q6), the Healthchecks account (Q6), and the alert email (Q7). Records each name and the renewal or billing date.
  - Requirements: R1.4, R9.9, R11.5.
  - Depends on: T00.
  - Verify: the file has no value that looks like a key (`gitleaks` in `security.yml` passes on the commit).

## Wave 1: fixes and checks that need no server

- [x] **T02 [P] (20 min) Ignore rules for infra secrets and state**
  - Files: `.gitignore`.
  - Add: `infra/**/*.tfstate`, `infra/**/*.tfstate.*`, `infra/**/.terraform/`, `infra/**/*.tfvars` (with `!infra/**/*.tfvars.example`), `infra/ansible/inventory/hosts.yml`, `infra/ansible/inventory/group_vars/*secret*`, `*.age` identity files named `*.agekey` and `*identity*.txt`, `rclone.conf`, `*.env` under `deploy/` (but keep `*.env.example`). `.env`, `.env.*` and `secrets/` already exist and stay.
  - Requirements: R1.5, R5.5.
  - Depends on: T00.
  - Verify: `git check-ignore -v` on a fixture for each pattern; `git check-ignore` returns no match for `.kiro/`; `git status` shows no new ignored file that should be tracked.

- [x] **T03 [S] (30 min) Red tests for the deploy script**
  - Files: `deploy/vps/tests/counterspell-deploy.bats`, `deploy/vps/tests/stubs/{docker,curl,cosign,flock,logger,sudo}`.
  - Write bats tests with stub binaries that record their arguments. Cover:
    - a bad request (not `deploy`, `rollback` or `status`, or a bad digest format) is refused and no stub is called except `logger`;
    - `deploy` with a failing `migrate run` does not call `up -d` (gap G1, R7.3, R7.4, P3);
    - a failing readiness check restores the previous release and keeps the candidate out of `deploy.env` (R7.7, P6);
    - `rollback` swaps the release records, runs no `migrate`, and refuses when no previous record exists (R7.9, P4);
    - a second run while the lock is held is refused (R7.11, P5);
    - a compose checksum mismatch refuses before any `pull` (R7.13, P7);
    - the `status` verb prints only the SHA and digests (R7.14).
  - Run the tests against the current script. They must fail for the expected reasons (G1 and missing verbs). Record the red output in the task note.
  - Requirements: R7, P3 to P7.
  - Depends on: T00.
  - Verify: `bats deploy/vps/tests/` runs and fails in the expected places; ShellCheck runs on the test helpers.

- [x] **T04 [S] [SUPERVISED] (30 min) Make the deploy script green**
  - Files: `deploy/vps/counterspell-deploy` (edit), `deploy/vps/counterspell-deploy-entry` (new, forced-command entry point).
  - Fix G1 (explicit error returns in `up`), G2 (readiness through `docker compose exec -T caddy`, then public health over HTTPS), G3 (`rollback` verb), G5 (`judge preflight` before judge starts), G6 (compose checksum check), and G7 (the entry point validates the verb and digests, then calls `sudo -n /usr/local/sbin/counterspell-deploy` with fixed arguments; the root script re-validates).
  - Merge `host.env` into the candidate `deploy.env` (R5.4). Keep the lock and the release records (R7.8, R7.11).
  - Requirements: R3.9, R6.6, R7.1 to R7.14, P3 to P7.
  - Depends on: T03, T12 (the `compose.sha256` layout).
  - Verify: `bats deploy/vps/tests/` all green; `shellcheck` clean on both scripts; `bash -n` on each; the regex in the entry point accepts exactly the grammar in R7.2 and R7.9.
  - Note: the `sudoers` line is written by T11, not here.

- [x] **T05 [P] (25 min) Correct the deploy notes**
  - Files: `deploy/vps/README.md`.
  - Replace step 1 (`docker` group) with the sudo rule (Q10, D5). Change the environment branches to `main` and `v*` (Q8, D6). Replace "Ansible will own this later" with a pointer to the `infra/ansible` roles. Point the key step at `counterspell-deploy-entry`. Keep the GitHub steps, the forced-command explanation and the host-key rule.
  - Requirements: R7.12, R7.15.
  - Depends on: T00.
  - Verify: `grep -n "kiro-setup\|docker\" group"` returns nothing in `deploy/vps/README.md`; the README's commands match the scripts' verbs.

- [x] **T06 [P] [S] (30 min) Readiness route `GET /api/ready` in the server**
  - Files: `crates/server/src/health.rs` (new, or the module that holds `/api/health`), `crates/server/src/lib.rs` (route only), `crates/server/tests/health_ready.rs`.
  - Add `/api/ready`. Ping Postgres and Valkey with `tokio::time::timeout` of 2 seconds each. Return `200 {"status":"ok"}` when both answer, or `503 {"status":"unavailable"}` otherwise. Log the failing dependency name at `warn`, never the URL. Keep `/api/health` unchanged (R10.1).
  - Tests: both up returns 200; Postgres down returns 503 and the body contains no connection string; Valkey down returns 503; a dependency that hangs past 2 seconds returns 503 in under 3 seconds (use a fake dependency with a paused clock).
  - Requirements: R10.1, R6.6, P12.
  - Depends on: run-lifecycle T20 (done), repo-skeleton T08 (done).
  - Verify: `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test -p counterspell-server`; the existing health and static tests still pass.
  - Rust lesson: `tokio::time::timeout` returns a `Result` that wraps the future's `Result`; why the 503 body is a fixed string (errors must not leak to clients).

- [x] **T07 [P] [S] (30 min) Judge `healthcheck` subcommand**
  - Files: `crates/judge/src/main.rs` (subcommand only), `crates/judge/src/health.rs` (new), `crates/judge/tests/healthcheck.rs`.
  - `counterspell-judge healthcheck` reads the worker heartbeat (run-lifecycle T17; Q26) and exits `0` when it is newer than 60 seconds, or `1` otherwise. It prints one line with no connection string.
  - Tests: fresh heartbeat exits 0; stale heartbeat exits 1; missing heartbeat exits 1; the subcommand never starts the worker loop.
  - Requirements: R10.2, R4.4.
  - Depends on: run-lifecycle T17 and T19 (done).
  - Verify: `cargo test -p counterspell-judge`; clippy clean; `docker compose config` (T15) lists the healthcheck.
  - Rust lesson: `std::process::ExitCode` and why `main` returns it instead of calling `exit` in the middle of the code; how `Result` in `main` prints the error.

- [x] **T08 [P] (30 min) Caddy edge: site address, headers, readiness block**
  - Files: `Caddyfile`. (Shared file. `auth` T33 changes it too; coordinate, and merge after auth T33.)
  - Site address `{$CADDY_SITE_ADDRESS::80}` so local use stays on port 80 (R6.1). Keep the SSE route unbuffered (R6.4). Add `respond /api/ready 404` before the proxy (R6.6). Add the headers of R6.3 and remove `Server`. Keep `encode zstd gzip` on other routes. Add the query-string filter for the OAuth callback and verify paths in the access log, following `auth` T33 (R6.5). Q17 decides the CSP text.
  - Requirements: R6.1 to R6.6.
  - Depends on: repo-skeleton T13 (done), auth T33 (open), Q17.
  - Verify: `docker run --rm -v "$PWD/Caddyfile:/etc/caddy/Caddyfile:ro" -e CADDY_SITE_ADDRESS=:80 caddy:2.10.2-alpine caddy validate --config /etc/caddy/Caddyfile` passes; the same with `CADDY_SITE_ADDRESS=counterspell.example` passes; a route test that asserts the 404 for `/api/ready` and the SSE settings in the text (in `deploy/vps/tests/caddy_shape.sh`).

## Wave 2: infrastructure code, not applied

- [x] **T09 [S] (30 min) Provider-neutral host module contract**
  - Files: `infra/terraform/modules/host/{versions.tf,variables.tf,outputs.tf,main.tf}`.
  - Write the contract from `design.md` (variables `name`, `datacentre`, `plan`, `image`, `ssh_public_keys`; outputs `public_ipv4`, `ssh_port`). `main.tf` only calls the provider module through a variable-free placeholder; the OVH resource lives in T10.
  - Requirements: R1.1, R1.3.
  - Depends on: T00, T02.
  - Verify: `terraform -chdir=infra/terraform/modules/host fmt -check`; `terraform -chdir=infra/terraform/modules/host init -backend=false`; `validate` (the module alone may need the placeholder).

- [x] **T10 [S] (30 min) OVH provider implementation and the prod environment**
  - Files: `infra/terraform/providers/ovh/{versions.tf,main.tf,outputs.tf}`, `infra/terraform/envs/prod/{versions.tf,main.tf,outputs.tf}`, `infra/terraform/envs/prod/terraform.tfvars.example`, `infra/terraform/.tflint.hcl`.
  - Implement the contract with the OVH provider's VPS resource (`ovh_vps`), pinned. Plan code and OS configuration keys are read from the OVH catalogue at T23 and written into the `.tfvars.example` as placeholders, marked `[U]`. The environment calls the module with `source = "../../providers/ovh"`.
  - Requirements: R1.1, R1.2, R1.3, R1.4, R1.5, R1.6.
  - Depends on: T09.
  - Verify: `terraform fmt -check -recursive infra/terraform`; `terraform -chdir=infra/terraform/envs/prod init -backend=false` and `validate`; `tflint --recursive` in `infra/terraform`. No `.tfstate`, `.terraform/` or real `.tfvars` in the tree (`git status`).

- [x] **T11 [P] (30 min) Ansible skeleton and the `base` role**
  - Files: `infra/ansible/{ansible.cfg,requirements.yml,site.yml,inventory/hosts.example.yml,inventory/group_vars/all.yml}`, `infra/ansible/roles/base/{tasks/main.yml,templates/sshd_hardening.conf.j2,templates/authorized_keys_deploy.j2,templates/sudoers_deploy.j2,handlers/main.yml,defaults/main.yml}`.
  - `base`: users `brighton` (sudo with password), `deploy` (no password, no sudo except the script, not in `docker`), `sandbox` (UID 1001, locked, no keys). `sshd` hardening (R2.3). UFW: `limit` SSH, allow 80 and 443, default deny in (R2.4, Q11). unattended-upgrades without auto-reboot (R2.5). timesyncd (R2.7). Restricted `authorized_keys` for `deploy` from a public key variable, with `command=` pointing at `counterspell-deploy-entry` (R2.9). The sudoers line for `deploy` names only `/usr/local/sbin/counterspell-deploy`, with no wildcard (R7.12).
  - Requirements: R2.1 to R2.5, R2.7 to R2.9, R7.12.
  - Depends on: T00.
  - Verify: `ansible-lint infra/ansible`; `ansible-playbook -i infra/ansible/inventory/hosts.example.yml infra/ansible/site.yml --syntax-check`; `visudo -cf` on the rendered sudoers in a throwaway file (in a temporary directory, by a test script).

- [x] **T12 [P] (30 min) Ansible `docker` and `app` roles**
  - Files: `infra/ansible/roles/docker/**`, `infra/ansible/roles/app/{tasks/main.yml,templates/host.env.j2,defaults/main.yml}`, `infra/ansible/roles/app/tasks/compose_sha.yml`.
  - `docker`: Docker Engine and Compose plugin from Docker's apt repository, pinned versions, `daemon.json` with json-file limits (R4.8), `brighton` in `docker`. Compose version at least 2.24 (for `!reset`).
  - `app`: `/opt/counterspell` (0750), `/etc/counterspell` (0700), `host.env` (0644, non-secret keys of design §Host layout), copies the three compose and Caddy files from `counterspell_src`, writes `compose.sha256` (Q19). Asserts that `postgres.env`, `migrate.env`, `server.env` and `judge.env` exist with mode 0600 and fails if one is missing. Installs the `counterspell-*` scripts with mode 0755.
  - Requirements: R2.6, R4.8, R5.1, R5.4, R7.13.
  - Depends on: T11.
  - Verify: `ansible-lint`; `--syntax-check`; the checksum task produces the same file format that T04 reads (a bats test in `deploy/vps/tests/compose_sha.bats`).

- [x] **T13 [S] [SUPERVISED] (30 min) Ansible `sandbox` role and the digest pre-pull**
  - Files: `infra/ansible/roles/sandbox/**`, `deploy/vps/prepull.sh`, `deploy/vps/tests/prepull.bats`.
  - Role: `uidmap`, subuid and subgid ranges, `loginctl enable-linger sandbox`, rootless install (`dockerd-rootless-setuptool.sh install`) as `sandbox`, the `delegate.conf` drop-in (R3.2), the `user@1001.service` and `user-1001.slice` limits (R3.3, Q13), the `docker.service` user drop-in that reapplies the socket ACL for UID 10001 (R3.4), an AppArmor check that stops with a message (R3.8, Q12).
  - `prepull.sh` reads only enabled `image = "...@sha256:..."` lines from `languages/languages.toml`, pulls by digest on the sandbox socket, and compares the local digest (R3.6). Tests use a fake `docker` and a fixture `languages.toml` with a fake digest that is never pulled.
  - Requirements: R3.1 to R3.8.
  - Depends on: T11, judge-sandbox T03 (done), T02 (VPS re-run in T25).
  - Verify: `ansible-lint`; `--syntax-check`; `bats deploy/vps/tests/prepull.bats`; `shellcheck deploy/vps/prepull.sh`.
  - Note: the role never sets a host-wide sysctl (R3.8).

- [x] **T14 [S] (30 min) Red shape test for the production Compose**
  - Files: `deploy/vps/tests/compose_shape.sh`, `deploy/vps/tests/fixtures/{postgres,migrate,server,judge}.env.example` (placeholder values only, names matching design §Secrets), `deploy/vps/tests/fixtures/host.env.example`.
  - The script runs `docker compose -f docker-compose.yml -f deploy/vps/compose.prod.yml --env-file <fixture deploy.env> --env-file <host.env.example> config --format json`, then asserts with `jq`:
    - P1: only `judge` mounts a sandbox socket; no service mounts `docker.sock`;
    - P2: only `caddy` publishes ports, only 80 and 443; `judge`, `postgres`, `valkey`, `migrate` not on `edge`; `data` is `internal`;
    - P8: `judge` env file is `judge.env` and its keys are the allow-list in design §Secrets;
    - R4.6: no `POSTGRES_HOST_AUTH_METHOD` anywhere;
    - R4.4: `read_only`, `cap_drop` ALL, `no-new-privileges`, `pids_limit`, `mem_limit` on `server`, `judge`, `migrate`;
    - R4.1: every image has a digest (`@sha256:`);
    - R4.7: `server` depends on `migrate` with `service_completed_successfully`;
    - R4.10: no base-file host-variable interpolation survives in `judge`.
  - Run it now. It must fail on the current compose files (red), and the failures must match the gaps G9, G10, G13, G14, G15. Record the output.
  - Requirements: R4.1 to R4.10, R12.4, P1, P2, P8.
  - Depends on: T00, judge-sandbox T38 and auth T33 (both edit `docker-compose.yml`; this test may be written before them).
  - Verify: `bash deploy/vps/tests/compose_shape.sh` exits non-zero with the listed red reasons; `shellcheck` clean.

- [x] **T15 [S] [SUPERVISED] (30 min) Production Compose green**
  - Files: `docker-compose.yml` (base: rename `internal` to `data`, remove `POSTGRES_HOST_AUTH_METHOD` only in the prod override, Valkey args, digests), `deploy/vps/compose.prod.yml` (full override as in design §Production Compose), `deploy/vps/tests/compose_shape.sh` (no change to assertions).
  - Fix G9 to G15. Pin `caddy`, `postgres`, `valkey` by digest (read the digests on the day, record them in the file's comments with the read date). Set `edge` subnet `172.30.10.0/24`. Use `!reset` for every base entry that reads a host variable (R4.10). Set the `logging` anchor (R4.8) and healthchecks (`postgres`, `valkey`, `judge`).
  - Requirements: R4.1 to R4.10, R5.1 to R5.3, R5.7, R5.8.
  - Depends on: T14, auth T33 and judge-sandbox T38 (merge both before editing, because they edit the same base file).
  - Verify: `bash deploy/vps/tests/compose_shape.sh` green; `docker compose -f docker-compose.yml config` (base, with `.env` absent) still works for local development; `docker compose ... config` with the prod override shows no `build:` for the three app services.
  - Note: Compose `!reset` needs Compose 2.24 or later. Record the local `docker compose version` in the task note.

- [x] **T16 [P] (30 min) Nightly encrypted backup**
  - Files: `deploy/vps/backup/counterspell-backup` (script), `deploy/vps/backup/counterspell-backup.service`, `deploy/vps/backup/counterspell-backup.timer`, `deploy/vps/backup/backup.env.example`, `deploy/vps/tests/backup.bats`.
  - Script: `pg_dump --format=custom --no-owner --no-privileges` through `docker compose exec -T postgres`, piped into `age -r "$AGE_RECIPIENT"` into `dumps/counterspell-<UTC>.dump.age`; SHA-256 line in `manifests/`; prune to 7 newest after the new file is written (R8.2, R8.3); `rclone copy` to the bucket (R8.4); Healthchecks `/success` or `/fail` (R8.7). Timer at `01:30:00` UTC with `Persistent=true` (R8.1).
  - Tests with fakes: plaintext never written to disk (the fake `pg_dump` output goes to a pipe and the test checks no plaintext file exists); prune keeps exactly 7; a failed `age` keeps old dumps; a failed upload still keeps the local encrypted dump and signals `/fail`. Use an `age` key generated in the test's temporary directory and deleted after (R12.6).
  - Requirements: R8.1 to R8.7, P10.
  - Depends on: T00.
  - Verify: `bats deploy/vps/tests/backup.bats`; `shellcheck`; `systemd-analyze verify` on the unit files (on a Linux machine; if unavailable, record "not run" for that line).

- [x] **T17 [P] (30 min) Ops report and the read-only role**
  - Files: `deploy/vps/ops/counterspell-ops-report` (script), `deploy/vps/ops/sql/ops-role.sql`, `deploy/vps/ops/sql/ops-report.sql`, `deploy/vps/ops/counterspell-ops-report.{service,timer}`, `deploy/vps/tests/ops.bats`.
  - The SQL grants `cs_ops_ro` column-level SELECT only on `jobs(state, run_after, lease_expires_at, updated_at)` and `submissions(id, status, created_at, finished_at)` (R9.4). The report runs the six queries of design §Ops report, prints one JSON line to journald, pings Healthchecks, and pings `/fail` on the R9.6 thresholds.
  - Tests: bats with a fake `psql` that returns fixture rows, checking the thresholds and that the JSON holds counts only. A grep test asserts that no query reads `code`, `payload`, `stdout` or `stderr`. A test asserts the grant file names no other column.
  - Verify the grants against the real migration columns in a throwaway Postgres 17 container (`[HUMAN]` if no container is free, recorded as P11 `MODEL ONLY` until then).
  - Requirements: R9.4 to R9.6, P11.
  - Depends on: T00, run-lifecycle migrations (done). T17 confirms the column names from the real migration when it runs.
  - Verify: `bats deploy/vps/tests/ops.bats`; `shellcheck`; the SQL parses (`psql` syntax check on a throwaway database).

- [x] **T18 [P] (25 min) Monthly restore drill and its result template**
  - Files: `deploy/vps/restore-drill/counterspell-restore-drill`, `deploy/vps/restore-drill/counterspell-restore-drill.service`, `deploy/vps/restore-drills/TEMPLATE.md`, `deploy/vps/tests/drill.bats`.
  - The drill downloads the newest off-box file, checks SHA-256 against the manifest, decrypts it with the identity file on `/dev/shm` (removed afterwards, Q23), restores into a throwaway `postgres:17-alpine` container on the `data` network, compares counts of `users`, `submissions`, `jobs`, `run_events` and the migration version with the manifest, and writes `deploy/vps/restore-drills/<YYYY-MM>.md` from the template (R8.8 to R8.10). The template has fields for date, file name, SHA-256, restore time, counts, migration version, result and reviewer. No user data.
  - Tests with fakes: a checksum mismatch fails the drill; the identity file is removed on exit (trap); the result file has counts only.
  - Requirements: R8.8 to R8.10.
  - Depends on: T16.
  - Verify: `bats deploy/vps/tests/drill.bats`; `shellcheck`.

## Wave 3: CI, dependency updates and runbooks

- [x] **T19 [S] (30 min) Deploy checks workflow**
  - Files: `.github/workflows/deploy-checks.yml` (new). Actions pinned to the SHAs already used in `ci.yml` and `terraform.yml` (no new third-party action without Brighton's approval).
  - Jobs: ShellCheck on `deploy/vps/**/*.sh` and the scripts; bats for all `deploy/vps/tests/*.bats`; `compose_shape.sh`; `caddy_shape.sh` and Caddy validate in the pinned image; `ansible-lint` and `--syntax-check`; `git ls-files` secret-name check (no `*.env` other than `*.env.example`, no `*.tfstate`, no `*.tfvars`).
  - Permissions: `permissions: {}` at the top, `contents: read` per job. Not a required check until Brighton adds it to the **CI OK** aggregate (ci-cd.md); recorded in the task note.
  - Requirements: R12.1 to R12.5, P9.
  - Depends on: T03, T04, T08, T13, T14, T15, T16, T17, T18.
  - Verify: actionlint and zizmor (already in `security.yml`) pass on the new file; a dry run of the workflow is `[HUMAN]` on the first push to the integration branch.

- [x] **T20 [P] (25 min) Renovate for the production images and the Ansible digests**
  - Files: `renovate.json` (extend the file judge-sandbox T38 creates).
  - Add a regex manager for the `@sha256:` digests in `deploy/vps/compose.prod.yml` and `docker-compose.yml`. Digest-only updates, one pull request per refresh, no auto-merge (Q20, R11.2).
  - Requirements: R11.2.
  - Depends on: judge-sandbox T38.
  - Verify: `npx --yes --package renovate -- renovate-config-validator renovate.json` if Brighton allows the tool; otherwise a JSON parse and `[HUMAN]` check after the Renovate app is installed.

- [x] **T21 [P] (30 min) Release checklist, rollback steps and the incident runbook**
  - Files: `deploy/vps/RELEASE.md`, `deploy/vps/RUNBOOK.md`, `deploy/vps/OPERATIONS-LOG.md` (template with a dated first line).
  - `RELEASE.md`: the pre-release, deploy and post-release items; the expand-only migration rule (R7.10) with a review checklist; the rollback command and when to use it; the link to `.kiro/steering/release-checklist.md` items it covers (R7.16).
  - `RUNBOOK.md`: deploy key rotation (R11.1), patch cadence (R11.2), the sandbox-escape flow (R11.3, design diagram) with exact commands, the availability and freeze rules (R11.5, R11.6).
  - Requirements: R7.10, R7.16, R11.1 to R11.7.
  - Depends on: T04.
  - Verify: every command in both files appears in a script or in the design; a markdown link check passes; no secret value appears.

- [x] **T22 [P] (25 min) Monitoring and alert setup notes**
  - Files: `deploy/vps/MONITORING.md`.
  - Exact settings for the UptimeRobot keyword monitor (URL, keyword `"ok"`, 5-minute interval, alert contact) and the four Healthchecks checks (backup, ops report, certificate, restore drill, with periods and grace from design §External monitors), and the test for each alert. Records the plan terms check for Q6.
  - Requirements: R9.7 to R9.10.
  - Depends on: T16, T17.
  - Verify: each check name in the file matches the name that the script pings.

## Wave 4: host work (needs the Wave 2 code merged; Brighton runs these)

- [ ] **T23 [S] [HUMAN] [GATE] (30 min) Order the VPS**
  - Commands (on Brighton's laptop, after T10 is merged): export the OVH credentials in the shell from the OVH console (not written to a file); `terraform -chdir=infra/terraform/envs/prod init`; `terraform -chdir=infra/terraform/envs/prod plan -out=tfplan`; Brighton reads the plan; `terraform apply tfplan`. Record the plan code, datacentre, VAT line, monthly price and the public IPv4 in `deploy/vps/ACCOUNTS.md`. Delete `tfplan` after apply.
  - Requirements: R1.6, R1.7, R11.5.
  - Depends on: T01, T10, Q2.
  - Verify: `terraform -chdir=infra/terraform/envs/prod plan` shows no changes after apply; the host answers SSH on the new IPv4 with Brighton's key.

- [ ] **T24 [S] [HUMAN] (30 min) Base configuration and SSH hardening**
  - Commands: `cp infra/ansible/inventory/hosts.example.yml infra/ansible/inventory/hosts.yml` (gitignored), fill the IP; `ansible-playbook -i infra/ansible/inventory/hosts.yml infra/ansible/site.yml --tags base --ask-become-pass`. Then from a second terminal: `ssh brighton@<ip>` (key only), `ssh root@<ip>` (refused), `ss -tln`, `sudo ufw status verbose`, `timedatectl show -p NTPSynchronized`. Run the playbook again and record `changed=0` for the `base` role.
  - Record results in `deploy/vps/verification-report.md` (section "Host base").
  - Requirements: R2.1 to R2.5, R2.7, R2.9, P9, P14.
  - Depends on: T11, T23.
  - Verify: the evidence above; `stat -c '%a %U:%G' /etc/counterspell` is `700 root:root` after the `app` role runs in T25.

- [ ] **T25 [S] [HUMAN] [SUPERVISED] (45 min) Docker, app and rootless sandbox**
  - Commands: `ansible-playbook ... --tags docker,app,sandbox`. Then on the host, as Brighton: `docker info --format '{{.CgroupVersion}} {{.CgroupDriver}}'`; `sudo -u sandbox XDG_RUNTIME_DIR=/run/user/1001 DOCKER_HOST=unix:///run/user/1001/docker.sock docker info` (look for `rootless`); `cat /sys/fs/cgroup/user.slice/user-1001.slice/user@1001.service/cgroup.controllers` (must include `cpu`, `cpuset`, `io`, `memory`, `pids`); `getfacl /run/user/1001/docker.sock` (must show `user:10001:rw-`); the prepull run and `docker image inspect` digest equality; `counterspell-judge preflight` run once the judge image exists (after T30; record "not yet" if not).
  - Re-run the judge-sandbox T02 checks on this host (the facts it lists) and append them to `deploy/sandbox-verification.md` as a VPS section, marking anything not checked as NOT TESTED. Brighton supervises each command (R3.8 may stop here; Q12 applies).
  - Requirements: R3.1 to R3.8, P14.
  - Depends on: T12, T13, T24, judge-sandbox T02 (open; this task is its VPS run).
  - Verify: each check above is recorded with its command and output in `deploy/vps/verification-report.md`.

- [ ] **T26 [S] [HUMAN] (30 min) DNS, mail domain and OAuth consoles**
  - Steps (Brighton): create the A and AAAA records for `counterspell.<domain>` to the VPS (Q1); wait for propagation (`dig +short counterspell.<domain>`); verify the sender domain in Resend with the SPF, DKIM and DMARC records it lists; register `https://counterspell.<domain>/api/auth/oauth/github/callback` and `…/google/callback` in the two OAuth consoles (Q27).
  - Record the names and dates (no secret) in `ACCOUNTS.md`.
  - Requirements: R6.1, R9.9 (email path), auth R4 and R6.
  - Depends on: T23, Q1, Q27.
  - Verify: `dig`, Resend's domain status shows verified, OAuth consoles show the callback URLs.

- [ ] **T27 [S] [HUMAN] [SUPERVISED] (45 min) Secrets and Postgres roles**
  - Steps (Brighton, on the host, never in chat or git): create `postgres.env`, `migrate.env`, `server.env` and `judge.env` with `umask 077` (design §Secrets). Check `stat -c '%a %U:%G' /etc/counterspell/*.env` gives `600 root:root`. Start Postgres alone (`docker compose -f docker-compose.yml -f deploy/vps/compose.prod.yml up -d postgres valkey`). Run the run-lifecycle role bootstrap (its deploy notes, steps 1 to 3) with `cs_server` and `cs_judge` logins (Q18). Create `cs_ops_ro` with `deploy/vps/ops/sql/ops-role.sql`. Confirm `cs_ops_ro` cannot read `submissions.code`: `psql` as `cs_ops_ro`, `SELECT code FROM submissions LIMIT 1` must fail with a permission error.
  - Requirements: R5.1 to R5.8, P8, P11.
  - Depends on: T15, T17, T24, T26, run-lifecycle deploy notes.
  - Verify: the three checks above recorded in the verification report, with no password in the record.

- [ ] **T28 [S] [HUMAN] [SUPERVISED] (45 min) Off-box backup and the first restore**
  - Steps: on Brighton's laptop, generate an `age` identity; keep the private key in the password manager and an offline copy; copy only the public recipient to `/etc/counterspell/backup.env` (`AGE_RECIPIENT`). Create the bucket and a bucket-scoped application key (Q4), with a 30-day lifecycle rule (Q5). Write `/etc/counterspell/rclone.conf` (0600) by hand. Install the backup units (`ansible-playbook --tags backup`, added to `app` role if needed). Run `systemctl start counterspell-backup.service` once. Download the newest object on the laptop, check the SHA-256 and decrypt it with the laptop identity; record the row counts.
  - Requirements: R8.1 to R8.7, R5.5, P10.
  - Depends on: T16, T18, T24, T27.
  - Verify: the file decrypts on the laptop and the counts are recorded in the drill file.

- [ ] **T29 [HUMAN] (30 min) Uptime monitor and dead-man checks**
  - Steps: follow `deploy/vps/MONITORING.md` (T22): create the UptimeRobot monitor, the four Healthchecks checks, and the alert contact. Test each alert: pause the monitor and confirm the Healthchecks `/fail` email arrives when the backup check is forced to fail once (stop the timer, wait for the grace, then restart it). Record the time of each test.
  - Requirements: R9.7 to R9.9.
  - Depends on: T22, T28.
  - Verify: each alert email is received and recorded in `OPERATIONS-LOG.md`.

- [ ] **T30 [S] [HUMAN] [SUPERVISED] [GATE] (60 min) First production deploy**
  - Pre-checks: judge-sandbox T37 has passed on the VPS (its gate for any deploy, `brighton-decisions.md`); run-lifecycle T29 (SSE) and auth T33 are done (both are open at the time of writing, so this task waits for them); the core flow works on the host (the 25 Nov checkpoint is met); T19 CI checks are green on `main`.
  - Steps: create the `production` environment with a required reviewer, branches `main` and `v*` (Q8); add `DEPLOY_HOST`, `DEPLOY_PORT`, `DEPLOY_SSH_KEY` (the restricted key's private half), `DEPLOY_KNOWN_HOSTS` (host key read on the server console, per `deploy/vps/README.md`); set repository variable `DEPLOY_ENABLED=true`; trigger **Deploy** with the SHA of a green `main` commit; approve. Then on the host: `counterspell-deploy-entry status`, `docker compose ps`, `curl -fsS https://counterspell.<domain>/api/health`, and the ops report journal line.
  - Requirements: R7.1 to R7.15, R10.3, R4.1 to R4.9.
  - Depends on: T04, T14, T15, T19, T25, T27, T29, judge-sandbox T37 (gate), auth T33, run-lifecycle T29.
  - Verify: the status line shows the deployed SHA; the public health returns `"ok"`; the monitor reports up; the recorded result is in `OPERATIONS-LOG.md`.

- [ ] **T31 [S] [HUMAN] [SUPERVISED] (60 min) Rollback rehearsal and canary log test**
  - Steps: (1) deploy a second green SHA (or re-run an earlier one), then run `rollback` through the workflow or the entry point; confirm the previous SHA is back, `migrate` did not run (the script's log says so), and health passes. Run `rollback` again to return. (2) Canary (R9.2, P13): submit one Python program through the Run flow with the marker `CANARY-<random>` in its source, its output and a comment; then `docker compose logs --no-color server judge caddy | grep -c CANARY-<random>` must print `0`; also search the auth callback path for a fake `code=CANARY` query and confirm the Caddy log holds no query string. (3) Stop and evidence rehearsal (RUNBOOK steps 1 and 2 only), then restart the judge; no deletion.
  - Requirements: R7.7, R7.9, R9.2, R11.3, P4, P13.
  - Depends on: T30, T21.
  - Verify: the three results recorded with their commands and output.

- [ ] **T32 [HUMAN] (20 min) Schedule the monthly drill**
  - Steps: run the drill once by hand (`counterspell-restore-drill`, or on the laptop per Q23). Create the Healthchecks check for the drill (period 35 days, grace 3 days, T29). Put the first Sunday of each month in Brighton's calendar.
  - Requirements: R8.8, R9.8.
  - Depends on: T18, T29.
  - Verify: `deploy/vps/restore-drills/<first-month>.md` exists with a pass result.

- [x] **T33 [S] (25 min) Verification report**
  - Files: `deploy/vps/verification-report.md` (results only; each R-item marked `TESTED on host <date>`, `TESTED in CI`, `MODEL ONLY` or `NOT TESTED`, with evidence and the command).
  - Fill from the task notes of T03 to T32. Anything not run stays `NOT TESTED` and is named.
  - Requirements: R12.7, R12.8, all R-items.
  - Depends on: T31, T32.
  - Verify: every R-item in `requirements.md` appears once; no line claims a host guarantee without a host record.

- [ ] **T34 [HUMAN] (20 min) Freeze and window checkpoints**
  - Wed 2 Dec 2026 (feature freeze, SAST): confirm the monitor history, the last backup, the last drill, the release record and the OPERATIONS-LOG entries. Confirm changes after this date are security or data-safety fixes (R11.6).
  - Fri 29 Jan 2027, 23:00 SAST (21:00 UTC): confirm the site is up, the window has held, and Brighton's decision after the window is recorded (Q25). Confirm the VPS and domain billing dates.
  - Requirements: R11.5, R11.6.
  - Depends on: T33.
  - Verify: each checkpoint line recorded in `OPERATIONS-LOG.md` with the date and the result.

## Dependency summary and parallel waves

- **Wave 0:** T00 (gate) first. T01 after T00.
- **Wave 1:** after T00: T02, T03, T05, T08 (after auth T33 is merged), T06 and T07 (Rust, parallel). T04 after T03 and T12. T04 is listed in Wave 1 because it fixes existing defects, but it can only go green after T12.
- **Wave 2:** T09 after T00 and T02; T10 after T09; T11 after T00; T12 after T11; T13 after T11; T14 after T00 (red may be written before auth T33 and judge-sandbox T38 are merged); T15 after T14 and both of those merges; T16 and T17 after T00; T18 after T16.
- **Wave 3:** T19 after T03, T04, T08, T13 to T18; T20 after judge-sandbox T38; T21 after T04; T22 after T16 and T17.
- **Wave 4:** T23 after T01, T10 and Q2; T24 after T11 and T23; T25 after T12, T13, T24 and judge-sandbox T02; T26 after T23 and Q1, Q27; T27 after T15, T17, T24, T26; T28 after T16, T18, T24, T27; T29 after T22, T28; T30 after T04, T14, T15, T19, T25, T27, T29, judge-sandbox T37 (gate), auth T33 and run-lifecycle T29; T31 after T30 and T21; T32 after T18 and T29; T33 after T31 and T32; T34 after T33.

## Completion checklist

Nothing here may be ticked from reasoning alone. Each item is ticked from the evidence in `deploy/vps/verification-report.md`.

- [ ] T00 signed off, and Q1 to Q27 answered in `design.md`.
- [ ] `bats deploy/vps/tests/` green (P3 to P7, P10, P11 with its note).
- [ ] `compose_shape.sh` green on the production override (P1, P2, P8, R4.x).
- [ ] `caddy validate` passes in the pinned image with the site address from env (R6.1).
- [ ] `ansible-lint` and `--syntax-check` pass (R12.2); second run `changed=0` on the host (P14).
- [ ] `terraform fmt`, `validate` and TFLint pass in CI (R12.1); no state or tokens in git (P9).
- [ ] `/api/ready` and `counterspell-judge healthcheck` tests pass (P12, R10).
- [ ] Rootless daemon on the host shows delegation, the socket ACL and the pre-pull digest check (R3.2, R3.4, R3.6), with the judge-sandbox T02 checks re-run.
- [ ] First deploy through the forced command, and a rollback, both recorded on the host (R7.1, R7.9, P4).
- [ ] Canary marker absent from every container log (P13).
- [ ] One nightly backup, one encrypted off-box copy and one restore drill, all recorded (R8.1, R8.4, R8.8).
- [ ] Each alert route tested by a forced failure (R9.6, R9.8).
- [ ] `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass for T06 and T07.
- [ ] `verification-report.md` names every R-item as tested on host, tested in CI, model only, or not tested.
- [ ] The uptime checkpoint on 29 Jan 2027 is recorded (T34).

## Agent task notes (2026-12-01)

- **T08**: auth-sensitive query keys (`code`, `state`, `token`) are replaced with REDACTED in
  the access log via a `format filter` block (R6.5). caddy_shape.sh asserts the filter and
  passes `caddy validate` in caddy:2.10.2-alpine for both `:80` and a hostname.
- **T15**: `docker compose version` = v5.5.1 (>= 2.24, `!reset`/`!override` supported).
  compose_shape.sh green; base `docker compose -f docker-compose.yml config` works with no
  .env; prod render contains zero `build:` entries. Third-party digests recorded with the
  2026-11-16 read date.
- **T19**: `.github/workflows/deploy-checks.yml` added — ShellCheck, bats, compose_shape,
  caddy_shape + validate, ansible-lint + --syntax-check, secret-name scan. Only
  actions/checkout@v7.0.1 (SHA already pinned in terraform.yml) is used. NOT part of the
  required CI aggregate — Brighton adds it to "CI OK" in ci-cd.md when ready. First-run on
  this branch is the human dry run.
- **T20**: renovate.json gained a regex manager matching `image: name:tag@sha256:` in
  compose.prod.yml and docker-compose.yml (verified against the real postgres/valkey pins),
  grouped as "production image digests", automerge false. Validator run is [HUMAN] after the
  Renovate app is installed.
- **T33**: `deploy/vps/verification-report.md` written — every R-item marked; all host-level
  guarantees are MODEL ONLY or NOT TESTED because no VPS exists yet.
