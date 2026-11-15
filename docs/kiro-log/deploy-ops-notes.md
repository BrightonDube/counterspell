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
