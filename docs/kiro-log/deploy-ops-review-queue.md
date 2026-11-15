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
