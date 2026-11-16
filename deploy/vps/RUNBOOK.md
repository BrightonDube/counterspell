# Operations runbook

Routine and incident procedures for the VPS. Every command here is a script in `deploy/vps/` or a standard host command. No secret values appear in this file. Record each rotation, drill, deploy, rollback and incident in `OPERATIONS-LOG.md` (R11.7).

## Deploy key rotation (every 6 months, or at once if exposed) (R11.1)

The rotation adds the new key, tests it, switches the GitHub secret, then removes the old key. Deploys never stop.

1. On the laptop, generate a new key with a new comment (`ssh-keygen -t ed25519 -C counterspell-ci-<yyyy-mm>`). Keep the private half in the password manager.
2. Add the new public key to `deploy_public_keys` in `infra/ansible/inventory/group_vars/all.yml` next to the old one, and run the base role (`--tags base`). Each line keeps `restrict,command="/usr/local/sbin/counterspell-deploy-entry"`.
3. Test from the laptop with the new key: `ssh -i <new key> deploy@<host> status` prints the release record.
4. Update the `DEPLOY_SSH_KEY` secret in the GitHub `production` environment.
5. Run a deploy of the current SHA, or a no-op redeploy, to confirm.
6. Remove the old key from `deploy_public_keys`, and run the base role again.
7. Record the rotation, with the date and the new key's comment only.

If the key may be exposed, skip the wait: do steps 1 to 4, then remove the old key in step 6 the same day.

## Patch cadence (R11.2)

- Ubuntu security updates install automatically (`unattended-upgrades`), without a reboot.
- Once a month, Brighton reviews the Docker, Compose, Caddy, Postgres and Valkey advisories.
- Image digest updates come from Renovate as pull requests. Brighton reviews and merges them. They are never auto-merged.
- A reboot is scheduled by hand when the host reports that one is needed (the base role prints the message).

## Judge and sandbox checks

- Judge health: `docker compose -f /opt/counterspell/docker-compose.yml -f /opt/counterspell/compose.prod.yml exec judge /usr/local/bin/counterspell-judge healthcheck`.
- Stop the judge (no new runs are claimed): `docker compose -f /opt/counterspell/docker-compose.yml -f /opt/counterspell/compose.prod.yml stop judge`.
- Ops report, once by hand: `sudo /usr/local/sbin/counterspell-ops-report`.

## Incident: suspected sandbox escape (R11.3, R11.4)

Treat the host as possibly compromised if a sandbox escape is suspected. Follow these steps in order.

1. **Stop the judge.** `docker compose ... stop judge`. Nothing new runs in the sandbox.
2. **Preserve evidence before any clean-up.** Do not delete containers, images or logs yet. Create one encrypted archive and a manifest:

   ```
   sudo install -d -m 0700 /var/backups/counterspell/evidence
   sudo tar -C / -cf - var/log/journal etc/counterspell/host.env opt/counterspell/deploy.env opt/counterspell/deploy.env.previous 2>/dev/null \
     | age -r "$AGE_RECIPIENT" > /var/backups/counterspell/evidence/incident-<yyyymmddThhmmZ>.tar.age
   sudo sh -c 'cd /var/backups/counterspell/evidence && sha256sum incident-*.tar.age > incident-<yyyymmddThhmmZ>.manifest'
   ```

   Then `docker inspect` and `docker logs` of the judge and sandbox containers, written to a file in the same evidence directory.
3. **Contain.** Stop the sandbox daemon for the `sandbox` user: `sudo -u sandbox XDG_RUNTIME_DIR=/run/user/1001 systemctl --user stop docker.service`. Stop the stack if it is still reachable from the attacker's path: `docker compose ... stop`. Block SSH from other addresses with UFW if needed.
4. **Copy the evidence off the host.** Move the archive and its manifest to the laptop and to the object-storage bucket, under the incident prefix.
5. **Rotate every credential the host could reach.** The deploy key (above), the Postgres role passwords (`cs_server`, `cs_judge`, `cs_ops_ro`), the `rclone` bucket key, the Healthchecks URLs, the Resend key, the OAuth client secrets and the LLM provider keys. Change each value in its `/etc/counterspell/*.env` file or the provider console, then restart the service that reads it.
6. **Rebuild, do not repair in place** when host root may be lost. Rebuild from Terraform and Ansible (T23 to T26), restore the newest passing backup (`counterspell-restore-drill` shows the checks), and deploy the release recorded in `deploy.env`.
7. Record the incident, the evidence file names, the rotations and the rebuild in `OPERATIONS-LOG.md`.

## Availability and freeze (R11.5, R11.6)

- The site stays online and free for reviewers until 23:00 SAST on 29 January 2027. Brighton records his decision for after that date in `OPERATIONS-LOG.md`.
- After the feature freeze on 2 December 2026, only security and data-safety fixes ship. Each gets a release record.
