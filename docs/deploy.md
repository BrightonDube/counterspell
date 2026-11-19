# Deploy runbook

Production is the shared OVH VPS (`15.204.92.66`, site `https://counterspell.online`). It shares
the box with Evorayn: never restart Caddy (validate + reload only) and check
`https://api.evorayn.com/health` before and after any host change. Counterspell lives under
`/opt/counterspell` owned by `deploy-counterspell`, with its own Postgres and Valkey and a rootless
sandbox daemon for the judge.

## Normal deploy

Pushes to `main` deploy automatically: a green `CI` run (which publishes the signed images)
triggers `Deploy` via `workflow_run`. The job verifies the image signatures, pins them by digest,
and calls the forced command over SSH as `deploy-counterspell`:

```
ssh -i <ci key> -p <DEPLOY_PORT> deploy-counterspell@<DEPLOY_HOST> \
  "deploy <40-hex sha> <server digest> <judge digest> <migrate digest>"
```

The deploy script pulls by digest, runs judge preflight and migrations, starts the stack, checks
readiness and public health, and restores the previous release itself if a check fails. The
workflow then re-checks `https://counterspell.online/api/health` and runs `rollback` if it does
not answer `"ok"`. Deploys require the `DEPLOY_ENABLED=true` repository variable and the
`production` environment secrets (`DEPLOY_HOST`, `DEPLOY_PORT`, `DEPLOY_SSH_KEY`,
`DEPLOY_KNOWN_HOSTS`).

Manual deploy of a specific green commit: Actions → Deploy → Run workflow, optionally passing the
full SHA.

## Rollback

Ask for the previous release over the same forced command (`status` shows current digests):

```
ssh -i counterspell-ci -p <port> deploy-counterspell@15.204.92.66 status
ssh -i counterspell-ci -p <port> deploy-counterspell@15.204.92.66 rollback
```

`rollback` swaps the current and previous release records and restarts the previous digests; it
never runs migrations (expand-only, see `deploy/vps/RELEASE.md`). Deploying an older green SHA is
an equivalent manual rollback. If the Caddy site change is at fault, remove the Counterspell block
from the shared Caddyfile and reload — never restart the Caddy container.

## When to re-run Ansible

Re-run `ansible-playbook -i inventory/hosts.yml counterspell-shared.yml
--vault-password-file ~/.config/counterspell/vault-pass` (from `infra/ansible/`) only when host
configuration changes — env/secrets templates, the forced-command key list (`deploy_public_keys`),
the sandbox AppArmor or rootless setup, or the Caddy site block. Image and code changes never need
Ansible; they ship through the deploy path above. The playbook runs only the `app`, `sandbox` and
`caddy_site` roles — never `base` or `docker` — and is guarded by `shared_host`.

Rotating a secret: change it in the vault, re-run Ansible to rewrite the env file, then re-deploy
(or `ssh … "status"` to confirm the release record is intact).
