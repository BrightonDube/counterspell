# deploy-ops task notes

Running log of task evidence. Blockers and Brighton-only steps go in `deploy-ops-blockers.md`.

## T02 (16 Nov 2026)

Red: `git check-ignore` on 11 fixtures returned no match for each pattern (exit 1).
Green: rules added to `.gitignore`; all must-ignore fixtures match, the four must-track paths do not match (exit 0). `gitleaks` on the commit: no leaks.

## T03 (16 Nov 2026)

Tools: bats 1.13.0 (`~/.local/node_modules`, not a repo dependency), shellcheck 0.11.0 (pip `shellcheck-py`, user install). Neither is a repo dependency.

Red output of `bats deploy/vps/tests/counterspell-deploy.bats` against the current script:

```
ok 1 a request that is not deploy, rollback or status is refused and no tool runs
ok 2 a deploy with a malformed digest is refused before any tool runs
not ok 3 a failing migrate stops the deploy before the stack is started (G1, R7.3, R7.4, P3)
ok 4 a failing readiness check restores the previous release and keeps the candidate out of deploy.env (R7.7, P6)
not ok 5 rollback swaps the release records and runs no migration (R7.9, P4)
ok 6 rollback refuses when there is no previous release record (R7.9, P4)
ok 7 a second deploy while the lock is held is refused (R7.11, P5)
not ok 8 a compose file that differs from compose.sha256 is refused before any pull (R7.13, P7)
not ok 9 status prints only the SHA and the digests, never host values (R7.14)
```

Expected reasons: test 3 fails because a failed `migrate run` still reaches `up -d` (G1). Test 5 fails because there is no `rollback` verb (G3). Test 8 fails because there is no compose checksum check (G6). Test 9 fails because `status` prints the whole file, including host keys (R7.14). Tests 4, 6 and 7 already pass against the current script; they stay as regression guards.

Stub note: `sleep` is also stubbed (not named in the task) so that health retries finish at once.

## T06 (16 Nov 2026)

Red: `cargo test -p counterspell-server --test health_ready` failed to compile (`counterspell_server::health` unresolved).
Green: 4 readiness tests and the existing health and static tests pass (`cargo test -p counterspell-server`, all green). `cargo fmt --check` and clippy with `-D warnings` pass on the workspace. The 2-second timeout test runs in real time (about 2 s), because `tokio` `test-util` (a paused clock) would add a feature the task does not name.

## T07 (16 Nov 2026)

Q26 check: run-lifecycle T17 renews leases in Postgres and writes no worker heartbeat row, so the Postgres option does not work for an idle worker. Used the Q26 fallback: a heartbeat file, written every 10 s by its own task in `service::serve` (not from the claim loop, which can block on a full slot set during long runs). The path comes from `JUDGE_HEARTBEAT_FILE`, default `/tmp/counterspell-judge.heartbeat` (tmpfs in the judge container).

Red: `cargo test -p counterspell-judge --test healthcheck` failed (no subcommand; the binary tried to start the worker and stopped on the missing `DATABASE_URL`).
Green: 3 integration tests (`report` in-process) and 3 unit tests in `health.rs`; judge suite 347 passed, 0 failed; clippy and fmt clean.
Not yet verified: `docker compose config` shows the healthcheck (needs T15). T07 stays unticked until then.
Decision: tests call `health::report` in-process, not the binary, because `crates/judge/clippy.toml` forbids `Command::new` in judge code.

## T08 (16 Nov 2026), partly done, not ticked

Red: `bash deploy/vps/tests/caddy_shape.sh` failed 10 checks (site address, readiness 404, headers, `-Server`).
Green: all 13 shape checks pass; `caddy validate` passes for `CADDY_SITE_ADDRESS=:80` and `counterspell.example` with the local `caddy:2.10.2-alpine` image (`--pull=never`, no registry call).
Still open:
- R6.5 access-log query-string filter for the OAuth callback and verify routes depends on auth T33 (open). The Caddyfile is shared with auth T33, so merge after it.
- Q17: the Content-Security-Policy is report-only. The SvelteKit build inlines a bootstrap script (`web/build/200.html`), so an enforced `script-src 'self'` would break the app. Enforcing needs a hash or nonce decision from Brighton. See `deploy-ops-blockers.md`.

## T09 and T10 (16 Nov 2026)

Tools (user installs, checksums checked, not repo dependencies): terraform 1.17.2 and tflint 0.66.0 from HashiCorp and the TFLint GitHub release.
Verification: `terraform fmt -check -recursive infra/terraform` passes; `init -backend=false` and `validate` pass for `modules/host`, `providers/ovh` and `envs/prod`; `tflint --recursive` passes (the contract's variables carry `tflint-ignore` comments because a contract has no consumer on its own).
Provider: `ovh/ovh` pinned to `= 2.25.0`. `ovh_vps` is created through OVH's order workflow, so `apply` places a billable order. Nothing here applies it (T23, Brighton only). `prevent_destroy` is set on the VPS.
Gap: the `ovh_vps` resource does not export `public_ipv4`. Output is null, and Brighton records the IP at T23.
Placeholders in `envs/prod/terraform.tfvars.example` are all marked `[U]` and need the OVH catalogue (T23).
No `.tfstate` or real `.tfvars` exists in the tree.

## T11 (16 Nov 2026)

Tools (user installs, not repo dependencies): ansible-core 2.22.0, ansible-lint 26.11.0 (pip, user). Collections pinned in `infra/ansible/requirements.yml`: community.general 12.5.0 and ansible.posix 2.1.0.
Environment note: `/mnt/c` is world-writable, so Ansible ignores `ansible.cfg` there. Set `ANSIBLE_CONFIG` and `ANSIBLE_COLLECTIONS_PATH` explicitly.
Verification: `ansible-playbook -i inventory/hosts.example.yml site.yml --syntax-check` passes; `ansible-lint .` passes (production profile, 0 failures); `infra/ansible/tests/check-base-render.sh` passes (visudo on the rendered sudoers, exact deploy path, no wildcard, forced-command line, sshd drop-in).
Not run: the role on a real host (T24, Brighton). The hardening values are checked by render, not on sshd.

## T12 (16 Nov 2026)

Verification: `ansible-lint infra/ansible` passes (production profile); `--syntax-check` passes; `bats deploy/vps/tests/compose_sha.bats` 3 of 3 (the rendered record passes the same `sha256sum --check` that counterspell-deploy uses, and a changed file fails it).
Role variables use the `app_` prefix (`app_src`, `app_dir`, `app_env_dir`) to pass ansible-lint.
Env-file checks are tagged `env-check`. Before T27, run the app role with `--skip-tags env-check`, as site.yml's comment says.
[U] Docker versions in `roles/docker/defaults/main.yml` (`docker-ce 5:28.0.0`, `docker-compose-plugin 2.24.0`) are placeholders. Brighton confirms them at T24 with `apt-cache madison`. A wrong pin stops the run at install time, and nothing else changes.
`host.env` needs `caddy_site_address` in group_vars (a public hostname, set at T26 with Q1). The role asserts it is set.

## T16 (16 Nov 2026)

Verification: `bats deploy/vps/tests/backup.bats` 6 of 6 pass (success with manifest and counts and success ping; plaintext marker never on disk; prune keeps exactly 7; failed `age` keeps old dumps and writes no partial; failed upload keeps the local dump and pings fail; missing recipient stops before any database call). `shellcheck` clean on the script and stubs.
`systemd-analyze verify` on the two units: the only message is that `/usr/local/sbin/counterspell-backup` is not installed on this machine. That path exists after the app role installs the script (T28), so the units verify on the host.
Deviation: the test uses a fixture recipient string and an `age` stub, not a real age key generated in the test directory. No `age-keygen` is installed here. The real round trip is done on Brighton's laptop at T28 (decrypt and checksum), which is the check that matters.

## T17 (16 Nov 2026)

Verification: `bats deploy/vps/tests/ops.bats` 9 of 9 pass; `shellcheck` clean on the report script and stubs.
Stub fix: the `logger` stub now records only the message after `--`, as real `logger` takes it. The earlier version recorded the `-t` option text too, and that broke the counts-only check.
SQL not run on a server (P11, MODEL ONLY). See `deploy-ops-blockers.md` for the kill-verdict grant decision.

## T18 (16 Nov 2026)

Verification: `bats deploy/vps/tests/drill.bats` 4 of 4 pass; `shellcheck` clean.
Safety fix found while testing: the cleanup trap removed `DRILL_IDENTITY_FILE` on every exit, including the refusal path. A wrong path outside `/dev/shm` would have deleted an arbitrary file. The trap now removes only files under `/dev/shm`, and the test `identity path outside /dev/shm is refused and the file is never deleted` checks it.
Test hygiene: the teardown now uses a separate `TEST_IDENTITY` variable, so no `/dev/shm` identity fixture is left behind.
Not run: the real Postgres 17 restore (needs the data network and a real dump). The round trip is T28 on the host and on Brighton's laptop.

## T21 and T22 (16 Nov 2026)

Docs written: `deploy/vps/RELEASE.md`, `deploy/vps/RUNBOOK.md`, `deploy/vps/OPERATIONS-LOG.md` (dated template), `deploy/vps/MONITORING.md`.
Checks: each command in these files is a script verb or a standard host command, and matches the scripts and design. The files hold no secret value (a scan for password, token and secret words found only instructions). No Markdown links to check.
T22 caveat: the scripts ping Healthchecks by URL, so the check names live only in `MONITORING.md`. Name-to-script matching is by URL, which is recorded in the file.
Gaps stated in MONITORING.md: the certificate check and the drill ping do not exist in code yet.
