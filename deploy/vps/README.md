# Deploying to the VPS (not live yet)

`.github/workflows/deploy.yml` is a **no-op** until all of this exists. It uses a GitHub-hosted runner, no
runner on the server, environment-scoped secrets, a pinned host key, and a forced-command key that cannot
open a shell.

The host is set up by the Ansible roles in `infra/ansible` (base, docker, app, sandbox). Run them as
described in `.kiro/specs/deploy-ops/tasks.md` (T24 and T25). The steps below are what those roles leave
for you to check by hand.

## On the server

1. The `deploy` user is created by the `base` role. It has no password, no shell login, and no `docker`
   group membership. Its only sudo right is one line that runs the deploy script (no wildcards):

   ```
   deploy ALL=(root) NOPASSWD: /usr/local/sbin/counterspell-deploy
   ```

   The forced command below is the first control. The sudo rule is the second: a leaked CI key can only
   run the deploy script with the arguments that the entry point allows.
2. The `app` role installs `counterspell-deploy` and `counterspell-deploy-entry` into `/usr/local/sbin`
   (root-owned, mode 0755). It copies `docker-compose.yml`, `compose.prod.yml` and `Caddyfile` to
   `/opt/counterspell` and writes `compose.sha256`. The deploy script refuses to run if those files differ
   from the checksum file.
3. Generate a dedicated key on your laptop: `ssh-keygen -t ed25519 -C counterspell-ci -f counterspell-ci`.
   Put the public half in `/home/deploy/.ssh/authorized_keys` as a single line. The `base` role templates
   this line from a variable:

   ```
   restrict,command="/usr/local/sbin/counterspell-deploy-entry" ssh-ed25519 AAAA... counterspell-ci
   ```

   `restrict` turns off port, agent and X11 forwarding and PTYs. `command=` forces `counterspell-deploy-entry`,
   which reads the client's request from `SSH_ORIGINAL_COMMAND`. It accepts only `deploy <sha> <digest>
   <digest> <digest>`, `rollback` and `status`, then calls the deploy script through sudo
   ([sshd(8)](https://man.openbsd.org/sshd.8#AUTHORIZED_KEYS_FILE_FORMAT)).
4. Read the host key **on the server console** (`cat /etc/ssh/ssh_host_ed25519_key.pub`) and build the
   known_hosts line yourself (`<host> ssh-ed25519 AAAA...`). Do not trust a first `ssh-keyscan`.

## Verbs

| Request | Effect |
|---|---|
| `deploy <40-hex sha> <server digest> <judge digest> <migrate digest>` | Pulls by digest, runs judge preflight and migrations, then starts the stack. Restores the previous release if a check fails. |
| `rollback` | Swaps the current and previous release records, starts the previous digests, runs the same checks. Never runs migrations. |
| `status` | Prints the SHA and the three digests of the current release. |

## On GitHub

1. Settings → Environments → **production**: required reviewer = you; deployment branches = `main` and tags
   `v*`. Required reviewers on environments are free for public repos
   ([docs](https://docs.github.com/en/actions/reference/workflows-and-actions/deployments-and-environments)).
2. Environment secrets: `DEPLOY_HOST`, `DEPLOY_PORT`, `DEPLOY_SSH_KEY` (private half), `DEPLOY_KNOWN_HOSTS`.
3. Make the three GHCR packages public (Packages → package settings), or the server cannot pull them anonymously.
4. Repository variable `DEPLOY_ENABLED=true`. Then Actions → Deploy → Run workflow.
