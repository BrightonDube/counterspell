# Deploying to the VPS (not live yet)

`.github/workflows/deploy.yml` is a **no-op** until all of this exists. It follows the same design as
the BizPilot CI/CD review: GitHub-hosted runner, no runner on the server, environment-scoped secrets,
a pinned host key and a forced-command key that cannot open a shell.

## On the server (Ansible will own this later)

1. Create a `deploy` user with no password and no sudo. Put it in the `docker` group.
   That group is root-equivalent; what the forced command buys you is that a leaked CI key can only
   redeploy images that are already signed in GHCR, not run arbitrary commands.
2. Install `counterspell-deploy` as `/usr/local/sbin/counterspell-deploy` (owner root, mode 0755) and
   copy `docker-compose.yml`, `compose.prod.yml` and `Caddyfile` to `/opt/counterspell` (owner root,
   group `deploy`, writable by `deploy` only for `deploy.env*` and `.deploy.lock`).
3. Generate a dedicated key on your laptop: `ssh-keygen -t ed25519 -C counterspell-ci -f counterspell-ci`.
   Put the public half in `/home/deploy/.ssh/authorized_keys` as a single line:

   ```
   restrict,command="/usr/local/sbin/counterspell-deploy" ssh-ed25519 AAAA... counterspell-ci
   ```

   `restrict` turns off port, agent and X11 forwarding and PTYs; `command=` forces the script and hands it
   the client's request in `SSH_ORIGINAL_COMMAND` ([sshd(8)](https://man.openbsd.org/sshd.8#AUTHORIZED_KEYS_FILE_FORMAT)).
4. Read the host key **on the server console** (`cat /etc/ssh/ssh_host_ed25519_key.pub`) and build the
   known_hosts line yourself (`<host> ssh-ed25519 AAAA...`). Do not trust a first `ssh-keyscan`.

## On GitHub

1. Settings → Environments → **production**: required reviewer = you; deployment branches =
   `brighton/kiro-setup` and tags `v*`. Required reviewers on environments are free for public repos
   ([docs](https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments)).
2. Environment secrets: `DEPLOY_HOST`, `DEPLOY_PORT`, `DEPLOY_SSH_KEY` (private half), `DEPLOY_KNOWN_HOSTS`.
3. Make the three GHCR packages public (Packages → package settings), or the server cannot pull them anonymously.
4. Repository variable `DEPLOY_ENABLED=true`. Then Actions → Deploy → Run workflow.
