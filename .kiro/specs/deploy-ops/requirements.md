# Requirements: deploy-ops

## Purpose

Take a signed image from GHCR to a running, backed-up and monitored Counterspell on one VPS. The service must stay online and free until 23:00 SAST on 29 Jan 2027 (`docs/system-design.md` §10, `docs/requirements.md` §7). The design names OVH VPS-2 as the first host, but the provisioning boundary must allow another provider later.

This spec builds on the deploy design that already exists. It keeps `.github/workflows/deploy.yml`, the forced-command SSH key and `deploy/vps/counterspell-deploy`. It changes those files only where the design review found a defect (see `design.md`, "Gap analysis").

## Status and scope

- **Approved for implementation** by Brighton on 15 Nov 2026 10:26, for code, IaC and docs only. No purchases, no provisioning, no real servers, and no spending. See `.kiro/steering/brighton-decisions.md` and the Review decisions section in `design.md`. T01 and other account-creation or paid-service tasks stay blocked.
- **In scope:** `infra/` (Terraform and Ansible), the production Compose file, the Caddy edge, release and rollback, backups, observability, security operations and verification.
- **Out of scope:** application features, sandbox code (`judge-sandbox`), authentication logic (`auth`), payments, a second VPS, high availability, Kubernetes, and any paid service unless Brighton approves it.
- **Constraints:** no secret value enters the repository, a log, a spec or a chat message. No file named `.env` is read or written. The main Docker daemon and the sandbox daemon stay separate. Code, config, docs and specs use neutral wording for dates and deadlines.

## Glossary

- **Main daemon:** the Docker Engine that runs Compose (app services, Postgres, Valkey, Caddy).
- **Sandbox daemon:** the rootless Docker daemon, run as user `sandbox`, that runs only user-code containers.
- **Digest:** the `sha256:` content hash of an image. Deploys pin images by digest, never by tag.
- **Env file:** a root-owned file under `/etc/counterspell/` that holds one service's secrets.
- **Drill:** a restore of a backup into a throwaway database, followed by a written result.
- **Window:** the availability period that ends at 23:00 SAST on 29 Jan 2027.

## R1. Provider-neutral provisioning

1. THE SYSTEM SHALL define the host in `infra/terraform/` with a provider-neutral module, `modules/host`, whose input and output variables are the only interface that environments use.
2. THE SYSTEM SHALL implement the OVH VPS-2 only under `providers/ovh/`. A later provider SHALL be added as a sibling directory, with no change to the module contract.
3. THE SYSTEM SHALL pin the Terraform version and the provider versions in `versions.tf`.
4. THE SYSTEM SHALL read provider credentials only from environment variables that Brighton sets in his own shell.
5. THE SYSTEM SHALL NOT commit Terraform state, state backups, `*.tfvars` files that contain values, or provider tokens. Local state SHALL live outside the repository.
6. THE CI workflow SHALL run only `terraform fmt -check`, `init -backend=false`, `validate` and TFLint. `plan` and `apply` SHALL run only on Brighton's machine. `apply` SHALL run only after Brighton reads the plan output.
7. IF the OVH provider cannot order the chosen plan or image, THEN THE SYSTEM SHALL stop and record the blocker. It SHALL NOT pick a different plan without Brighton's approval.

## R2. Host configuration

1. THE SYSTEM SHALL configure the host with Ansible under `infra/ansible/`. A second run of the same playbook SHALL report no changes.
2. THE SYSTEM SHALL create three users:
   - `brighton`: administrator, key login only, sudo with a password;
   - `deploy`: no password, no sudo rights beyond the deploy script (R7.12), key login only, with the forced command;
   - `sandbox`: locked password, no authorized keys, owner of the rootless sandbox daemon.
3. THE SYSTEM SHALL disable root login and password authentication over SSH, allow only the `brighton` and `deploy` users, and set `MaxAuthTries 3`.
4. THE SYSTEM SHALL allow inbound traffic only on the SSH port (rate-limited), TCP 80 and TCP 443. Every other inbound port SHALL be denied by default.
5. THE SYSTEM SHALL install unattended security upgrades. It SHALL NOT reboot automatically. It SHALL report when a reboot is required.
6. THE SYSTEM SHALL install Docker Engine and the Compose plugin for the main daemon from Docker's official apt repository, at pinned versions.
7. THE SYSTEM SHALL keep time synchronised with systemd-timesyncd, and the preflight in R3.9 SHALL check that the clock is synchronised.
8. THE SYSTEM SHALL NOT receive or store application secrets through Ansible. Its variables SHALL contain only non-secret values, such as the site name and the public keys.
9. THE SYSTEM SHALL write the CI deploy public key into `authorized_keys` as a single line with the options `restrict,command=` pointing at the deploy entry point. The public key is not secret.

## R3. Rootless sandbox daemon

1. THE SYSTEM SHALL run the sandbox daemon as a systemd user service of `sandbox`, with its own data root and its own socket, separate from the main daemon.
2. THE SYSTEM SHALL install the drop-in `user@<uid>.service.d/delegate.conf` with `Delegate=cpu cpuset io memory pids`. Before any image is pulled, THE SYSTEM SHALL verify that the controllers are delegated (`cgroup.controllers` of the user manager).
3. THE SYSTEM SHALL cap the aggregate CPU, memory and tasks of the sandbox user's services with systemd properties (defaults: `MemoryMax=4G`, `TasksMax=512`, `CPUQuota=300%`).
4. THE SYSTEM SHALL give the sandbox socket to the judge container only. It SHALL do this with a POSIX ACL for the judge's UID on the socket and on its directory. No other UID, group or container gets access. The ACL SHALL be reapplied each time the daemon starts.
5. THE SYSTEM SHALL enable lingering for `sandbox`, so that its user service runs with no login session.
6. THE SYSTEM SHALL pull each enabled sandbox image by the digest pinned in `languages/languages.toml`, and SHALL check after the pull that the local image digest equals the pinned digest.
7. THE SYSTEM SHALL NOT let a sandbox image be pulled at run time.
8. IF Ubuntu's restriction on unprivileged user namespaces (AppArmor) stops rootless Docker from starting, THEN THE SYSTEM SHALL stop and record the options for Brighton. It SHALL NOT change the host-wide sysctl without his approval.
9. THE deploy script SHALL run `counterspell-judge preflight` (judge-sandbox T22) on every deploy, before the judge is started, and SHALL fail the deploy if preflight fails.

## R4. Production Compose topology

1. THE SYSTEM SHALL run these services: `caddy`, `server`, `judge`, `postgres` (17), `valkey` (8) and the one-shot `migrate`. Every third-party image SHALL be pinned by digest.
2. THE SYSTEM SHALL place `caddy` and `server` on the `edge` network, which has outbound internet access. It SHALL place `server`, `judge`, `postgres`, `valkey` and `migrate` on the `data` network, which SHALL be `internal: true`. Only `caddy` SHALL publish ports (TCP 80 and 443).
3. THE SYSTEM SHALL mount into `judge` only the sandbox socket. No Docker socket SHALL be mounted into any other service, and the main daemon socket SHALL NOT be mounted into any container.
4. THE SYSTEM SHALL run `server`, `judge` and `migrate` with a read-only root filesystem, `cap_drop: [ALL]`, `security_opt: [no-new-privileges:true]`, UID 10001, a PID limit, a memory limit and a CPU limit.
5. THE SYSTEM SHALL set these memory limits: `postgres` 1024m (with `shared_buffers=256MB`), `valkey` 160m (with `maxmemory 100mb`), `server` 256m, `judge` 256m, `caddy` 64m, and `migrate` 256m (only during a migration).
6. THE SYSTEM SHALL configure Postgres to accept TCP connections with `scram-sha-256` only. THE production file SHALL NOT set `POSTGRES_HOST_AUTH_METHOD`.
7. THE SYSTEM SHALL start `server` only after `migrate` has completed successfully, and start `judge` only after `postgres` is healthy.
8. THE SYSTEM SHALL cap the Docker json-file log of every service at 10 MB per file and 5 files.
9. THE SYSTEM SHALL give the `edge` network a fixed subnet and set `TRUSTED_PROXIES` to that subnet, so that per-IP limits trust only Caddy.
10. THE production file SHALL reset every base-file entry that reads a host variable, so that no secret reaches a service through interpolation.

## R5. Secrets and configuration

1. THE SYSTEM SHALL read secrets from root-owned files under `/etc/counterspell/`, one per service: `postgres.env`, `migrate.env`, `server.env` and `judge.env`. Each file SHALL have mode 0600.
2. `judge.env` SHALL contain only the judge's own database URL. It SHALL NOT contain OAuth, Resend, LLM, session or owner credentials.
3. `migrate.env` SHALL contain only the owner database URL. `server.env` SHALL contain the server's database URL, the Valkey URL and the application secrets named in `auth/design.md` (for example `SESSION_SECRET`, OAuth client secrets, `RESEND_API_KEY`).
4. THE SYSTEM SHALL keep non-secret host values in `/etc/counterspell/host.env`, which Ansible manages. The deploy script SHALL copy that file into each release record.
5. THE SYSTEM SHALL keep every env file, key and credential out of git. `.gitignore` SHALL cover them (task T02).
6. Brighton SHALL create and rotate each secret by hand on the host. Secrets SHALL NOT pass through Ansible, CI, chat or a repository file.
7. Each service SHALL use its own Postgres login. The server and judge logins SHALL be members of the NOLOGIN roles `counterspell_server` and `counterspell_judge` created by migrations (`deploy/run-lifecycle.md`). No service SHALL use the owner role at run time.
8. THE SYSTEM SHALL create the read-only operations login `cs_ops_ro` with column-level SELECT only, as in R9.4.

## R6. Edge, TLS and HTTP headers

1. THE SYSTEM SHALL serve the site over HTTPS, with certificates obtained and renewed by Caddy. The site address SHALL come from `CADDY_SITE_ADDRESS` in `host.env`. Without it, Caddy SHALL serve plain HTTP on port 80 for local use.
2. WHEN a plain HTTP request arrives on the public site, THE SYSTEM SHALL redirect it to HTTPS.
3. THE SYSTEM SHALL send these headers on every response: `Strict-Transport-Security`, `X-Content-Type-Options: nosniff`, `Referrer-Policy`, `Permissions-Policy`, `X-Frame-Options: DENY`, and a `Content-Security-Policy` agreed with the web build (Q17). It SHALL remove the `Server` header.
4. THE SYSTEM SHALL keep the run-event SSE route (`/api/runs/{id}/events`) unbuffered and uncompressed (`flush_interval -1`), as in `deploy/run-lifecycle.md`.
5. THE SYSTEM SHALL drop the query string from the access log for the OAuth callback and verify routes (`auth` T33).
6. THE SYSTEM SHALL NOT expose `/api/ready` to the internet. Caddy SHALL answer `404` for it. The deploy script SHALL check it from inside the stack.

## R7. Release and rollback

1. THE SYSTEM SHALL deploy only images that the publish workflow has signed and attested for a full commit SHA on `main` or a `v*` tag, as `.github/workflows/deploy.yml` already does.
2. THE SYSTEM SHALL pin every deploy by image digest, and SHALL NOT deploy an image by tag.
3. THE SYSTEM SHALL run the `migrate` container to completion before it starts or replaces `server` or `judge`.
4. IF a migration fails, THEN THE SYSTEM SHALL leave the running release unchanged and SHALL record the failure.
5. THE SYSTEM SHALL run `counterspell-judge preflight` (R3.9) before starting the judge.
6. THE SYSTEM SHALL check the new server's readiness from inside the stack (`/api/ready` through the `caddy` container) and check public health (`https://<site>/api/health`). It SHALL record the release as current only after both pass.
7. IF a readiness or health check fails after the new release starts, THEN THE SYSTEM SHALL restore the previous release automatically.
8. THE SYSTEM SHALL keep one previous release record (`deploy.env.previous`).
9. THE SYSTEM SHALL provide a `rollback` command through the same forced-command entry point. It SHALL swap the current and previous release records, pull and start those digests, and SHALL NOT run `migrate`.
10. THE SYSTEM SHALL write every migration so that the previous release still works against the new schema (expand, then contract). A release SHALL NOT drop, rename or change the type of a table or column that the previous release reads.
11. THE SYSTEM SHALL allow only one deploy or rollback at a time, using an exclusive lock. A second request SHALL be refused with a clear message.
12. THE SYSTEM SHALL let the `deploy` user run only the deploy script, through `sudo` with a rule that names the script and allows no arguments wildcard (Q10). The forced-command entry point SHALL validate the request before it calls sudo. The deploy user SHALL NOT be a member of the `docker` group.
13. THE SYSTEM SHALL refuse a deploy when the SHA-256 checksums of `docker-compose.yml`, `compose.prod.yml` and `Caddyfile` on the host differ from the checksums recorded by Ansible (`/opt/counterspell/compose.sha256`) (Q19).
14. THE SYSTEM SHALL answer `status` with the current release record only (SHA and digests). It SHALL NOT print any secret.
15. THE SYSTEM SHALL run deploys only from the `production` GitHub environment, which needs a required reviewer and restricts branches and tags to `main` and `v*`.
16. THE SYSTEM SHALL keep a release checklist in `deploy/vps/RELEASE.md` that covers the deployment items of `.kiro/steering/release-checklist.md`.

## R8. Backups and restore

1. THE SYSTEM SHALL run a nightly `pg_dump --format=custom` of the `counterspell` database at 01:30 UTC (03:30 SAST), from a systemd timer with `Persistent=true`.
2. THE SYSTEM SHALL encrypt every dump with `age` to a public recipient key before the dump is written to the host.
3. THE SYSTEM SHALL keep the 7 most recent encrypted dumps on the host. It SHALL delete an older dump only after the new dump and its SHA-256 manifest entry are written.
4. THE SYSTEM SHALL copy each encrypted dump to an object-storage bucket, using an application key that can access only that bucket.
5. THE SYSTEM SHALL keep the `age` private key off the server. The server SHALL NOT be able to decrypt existing backups.
6. THE SYSTEM SHALL set a 30-day lifecycle rule on the bucket (Q5).
7. IF any backup step fails, THEN THE SYSTEM SHALL keep the previous good dumps, and SHALL signal failure to the dead-man monitor (R9.8).
8. THE SYSTEM SHALL run a restore drill once a month. The drill SHALL download the latest off-box copy, decrypt it, restore it into a throwaway Postgres 17 container on the host, compare row counts and the migration version with the manifest, and write the result to `deploy/vps/restore-drills/<YYYY-MM>.md`.
9. IF a drill fails, THEN THE SYSTEM SHALL record the failure and treat backups as unverified until a later drill passes.
10. THE drill result files SHALL contain counts, hashes, timings and pass or fail. They SHALL NOT contain user data, email addresses, code or secrets.

## R9. Observability and alerts

1. THE SYSTEM SHALL run `server` and `judge` with `LOG_FORMAT=json` in their env files, so their logs are structured.
2. THE SYSTEM SHALL NOT log user source code, program stdout or stderr, hidden test inputs or outputs, OAuth codes, tokens, passwords, session values or keys (ADM-4). A canary test SHALL prove this on the host (T31).
3. THE SYSTEM SHALL rotate container logs with the limits in R4.8, and SHALL cap journald at 500 MB.
4. THE SYSTEM SHALL run an ops report every 5 minutes as a systemd timer. The report SHALL read through the `cs_ops_ro` login, which SHALL have SELECT on these columns only: `jobs(state, run_after, lease_expires_at, updated_at)` and `submissions(id, status, created_at, finished_at)`. The report SHALL contain counts and durations only.
5. THE ops report SHALL measure: queued job count, age of the oldest queued job, claimed jobs with expired leases, dead jobs in the last 24 hours, submissions ending in `output_limit`, `time_limit`, `memory_limit` or `internal_error` in the last hour, and p95 of (`finished_at` minus `created_at`) for submissions finished in the last hour.
6. IF the oldest queued job is older than 120 seconds, or any `internal_error` or dead job appears in the last hour, THEN THE ops report SHALL signal failure to the alert channel.
7. THE SYSTEM SHALL check `https://<site>/api/health` from outside the host with a free external uptime monitor, every 5 minutes, with a keyword check and email alerts.
8. THE SYSTEM SHALL use dead-man checks for the nightly backup, the ops report and the certificate check, so that a silent failure becomes an alert.
9. THE SYSTEM SHALL send every alert to Brighton's email address. It SHALL NOT use a paid pager or paid alert service.
10. THE SYSTEM SHALL check the certificate's expiry daily and signal when fewer than 14 days remain.

## R10. Health and readiness routes

1. THE server SHALL keep `GET /api/health` as a liveness route (repo-skeleton R6). It SHALL add `GET /api/ready`, which returns `200` only when Postgres and Valkey both answer within 2 seconds, and `503` with a generic body otherwise. The body SHALL NOT include a connection string or an error detail. (T06)
2. THE judge SHALL provide a `counterspell-judge healthcheck` subcommand. It exits `0` when its heartbeat is fresh (run-lifecycle T17), and non-zero otherwise. (T07)
3. THE Compose file SHALL use the judge healthcheck. The deploy script SHALL use `/api/ready`.

## R11. Security operations

1. THE SYSTEM SHALL rotate the deploy SSH key every 6 months, and at once if it may be exposed. Rotation SHALL add the new key, test it, switch the GitHub secret, then remove the old key, so that deploys never stop.
2. THE SYSTEM SHALL apply security updates automatically. Brighton SHALL review Docker, Compose, Caddy, Postgres and Valkey advisories each month. Image digest updates SHALL arrive as reviewed pull requests, never as auto-merged changes.
3. THE SYSTEM SHALL keep a written incident runbook for a suspected sandbox escape. The runbook SHALL: stop the judge; preserve evidence before any clean-up; contain; rotate the credentials that the host could reach; and rebuild from Terraform and Ansible. A rebuild is preferred to an in-place repair when host root may be lost.
4. THE SYSTEM SHALL store evidence as an encrypted archive with a SHA-256 manifest, copied off the host.
5. THE SYSTEM SHALL keep the site online and free until 23:00 SAST on 29 Jan 2027, and SHALL record Brighton's decision for after that time.
6. AFTER the feature freeze on 2 Dec 2026, THE SYSTEM SHALL allow production changes only for security fixes and data-safety fixes. Each change SHALL have a release record.
7. THE SYSTEM SHALL record every rotation, drill, deploy, rollback and incident in `deploy/vps/OPERATIONS-LOG.md`, without secret values.

## R12. Verification without a server

1. THE SYSTEM SHALL check Terraform with `fmt -check`, `validate` (after `init -backend=false`) and TFLint, in CI.
2. THE SYSTEM SHALL check Ansible with `ansible-lint` and `ansible-playbook --syntax-check`. Where feasible, it SHALL check the idempotence of the roles that do not need systemd in a throwaway Ubuntu container.
3. THE SYSTEM SHALL check every deploy and ops shell script with ShellCheck, and SHALL test it with bats using stub binaries (`docker`, `curl`, `cosign`, `ssh`, `pg_dump`, `age`, `rclone`, `flock`). No test SHALL call a real service.
4. THE SYSTEM SHALL check the production Compose configuration with `docker compose config`, and with a shape test that asserts R3, R4 and R5 (ports, networks, socket mounts, authentication method, digests, limits, env files).
5. THE SYSTEM SHALL validate the Caddyfile with `caddy validate` in the pinned Caddy image.
6. THE SYSTEM SHALL test backup, ops and drill scripts with a fake `pg_dump`, `rclone` and `age`. Any `age` key used by a test SHALL be generated in a temporary directory, used once and deleted.
7. WHERE a check needs the real host, THE SYSTEM SHALL list it as a `[HUMAN]` task with the exact commands and the result file to fill in.
8. THE SYSTEM SHALL NOT mark a host-level guarantee as tested until a `[HUMAN]` task has recorded the evidence.

## Acceptance summary

The spec is done when Brighton has answered the open questions in `design.md`, the tasks in `tasks.md` have the verification named in each task, and the verification report (T33) lists every R-item as `TESTED on host`, `TESTED in CI`, `MODEL ONLY`, or `NOT TESTED`, with evidence.

## Requirement sources

| Source | Used for |
|---|---|
| `docs/system-design.md` §2, §7, §10, §11, §13 | topology, sandbox controls, memory budget, cost, risks |
| `docs/requirements.md` §7, ADM-2, ADM-4 | availability, backups, monitor, logs without user code |
| `docs/ci-cd.md`, `.github/workflows/*.yml` | existing deploy and publish design |
| `deploy/vps/*`, `deploy/rootless-sandbox-daemon.md`, `deploy/run-lifecycle.md` | existing host and deploy notes |
| `.kiro/specs/judge-sandbox/` (T02, T22, T37, T38) | sandbox daemon, preflight, digests, renovate |
| `.kiro/specs/auth/` (T33, requirements R4, R9, R19) | domain, OAuth callbacks, Resend, log filter, env names |
| `.kiro/specs/repo-skeleton/` | crate names, `/api/health`, Caddy and Compose baseline |
| `.kiro/steering/security.md`, `sandbox.md`, `release-checklist.md` | security rules, sandbox boundary, release items |
