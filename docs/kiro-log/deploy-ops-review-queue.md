# deploy-ops review queue ([SUPERVISED] tasks)

Brighton reviews each item before the change is accepted on the host.

## T04: deploy script green (G1 to G7)

- Commit: see the T04 commit in `git log --oneline` (message "deploy-ops T04").
- Files: `deploy/vps/counterspell-deploy` (rewritten, argv interface), `deploy/vps/counterspell-deploy-entry` (new), `deploy/vps/tests/*`.
- Check:
  1. Failed `migrate` and failed `preflight` stop before `up -d` (G1).
  2. Readiness goes through `docker compose exec -T caddy` to `server:8080/api/ready`; the public check is HTTPS on `CADDY_SITE_ADDRESS` (G2).
  3. `rollback` swaps the records and runs no migration (G3, R7.9).
  4. `counterspell-deploy-entry` passes only fixed arguments to `sudo -n /usr/local/sbin/counterspell-deploy` (G7). The sudoers line (T11) must name that one path with no wildcard.
  5. Interface change: the root script now reads argv, because sudo clears `SSH_ORIGINAL_COMMAND`. The bats harness was updated to match. No assertion was weakened.
- Open dependencies: `counterspell-judge preflight` does not exist yet. The `run --rm judge preflight` step will fail until the judge gains that subcommand. This is intentional: the deploy refuses rather than skipping the check. Recorded in `deploy-ops-blockers.md`.

## T13: sandbox role and digest pre-pull (new, supervised)

- Files: `infra/ansible/roles/sandbox/**`, `deploy/vps/prepull.sh`, `deploy/vps/tests/prepull.bats`, `deploy/vps/tests/fixtures/languages.toml`, `deploy/vps/tests/stubs/docker` (image inspect branch).
- Check on the host (T25):
  1. `user@1001.service` has the `Delegate=cpu cpuset io memory pids` drop-in and the limits. The role asserts the controllers before any pull.
  2. The AppArmor check: the role stops with a message if `apparmor_restrict_unprivileged_userns` is 1. It never changes a host-wide sysctl (R3.8, Q12). Brighton decides any AppArmor change.
  3. The ACL drop-in grants `u:10001:rw` on the socket and `u:10001:x` on `/run/user/1001`, reapplied on each daemon start.
  4. `counterspell-prepull` pulls only enabled languages by digest, on the sandbox socket.
- Notes: the role names the Docker pins from `group_vars/all.yml` (`docker_ce_version`). Role defaults are scoped per role, so this moved there. `app_src` became `repo_root` for the same reason.
- Not run: the role on a real host (T25). The rootless install, the cgroup check and the ACL are host-only.
