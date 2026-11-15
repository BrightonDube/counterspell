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
