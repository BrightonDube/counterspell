# Separate rootless sandbox daemon

The sandbox is a separate security boundary from the main Docker Compose
daemon. User code, reference code and every code-running MCP request execute
only inside containers on the **rootless** daemon described here. The main
Docker daemon is never a sandbox boundary.

## Boundary

- Create an unprivileged operating-system user named `sandbox`.
- Run Docker as a systemd **user** service owned by `sandbox`.
- Use a rootless socket owned by that user, separate from the main Compose
  Docker daemon socket.
- Use the daemon only for throwaway user-code sandboxes. The main Docker
  daemon must never be treated as the sandbox boundary.
- The server, Caddy, Postgres, Valkey, and application secrets do not access
  either Docker daemon. In particular, the server has no Docker client or
  Docker socket.
- Only the trusted `counterspell-judge` worker receives access to the
  separate sandbox socket. The worker must not receive the main Docker daemon
  socket and must have no inbound port.
- The `sandbox` user's home holds nothing sensitive: no application secrets,
  no database credentials, no SSH material. Application secrets stay mode
  `0600`, owned by a different user, so a compromise inside the sandbox
  boundary never reaches them.

## Host setup

### cgroup v2 delegation

The judge applies CPU, memory, PID and IO limits through the container spec.
In rootless mode those limits only work if the `sandbox` user's systemd slice
has the controllers delegated. Check before enabling the judge:

```sh
loginctl enable-linger sandbox
systemctl --user -M sandbox@ show docker.service -p Delegate
cat /sys/fs/cgroup/user.slice/user-$(id -u sandbox).slice/cgroup.controllers
```

If `Delegate` does not already include `cpu cpuset io memory pids`, add the
drop-in:

```sh
mkdir -p ~sandbox/.config/systemd/user/docker.service.d
cat > ~sandbox/.config/systemd/user/docker.service.d/delegate.conf <<'EOF'
[Service]
Delegate=cpu cpuset io memory pids
EOF
chown -R sandbox:sandbox ~sandbox/.config
systemctl --user -M sandbox@ daemon-reload
systemctl --user -M sandbox@ restart docker
```

Re-run the two checks and record the output in `deploy/sandbox-verification.md`
before relying on the limits.

### Aggregate limits on the sandbox user's slice

Cap the whole sandbox workload — not just each container — so a burst of runs
cannot starve the host. Set the slice properties to values sized for the VPS
(example for a small host running `JUDGE_CONCURRENCY=2`):

```sh
systemctl set-property user-$(id -u sandbox).slice \
  CPUQuota=300% MemoryMax=8G TasksMax=4096
```

Persist the same limits in
`/etc/systemd/system/user-.slice.d/50-sandbox.conf` (drop-in for the per-user
slice) or an equivalent override so they survive reboot.

### Socket ownership

The rootless daemon listens at `/run/user/$(id -u sandbox)/docker.sock`,
owned `sandbox:sandbox`, mode `0660` by default. Access is granted by group
membership, never by mode `0666`:

- Put the judge's runtime user in a dedicated group (for example `sandbox-api`)
  and set `socket-group = sandbox-api` in the `sandbox` user's
  `~/.config/docker/daemon.json`, then restart the user daemon.
- Only the judge container joins that access path. No other container mounts
  the socket; the compose file mounts it into `judge` alone and the boundary
  test (`crates/judge/tests/boundary.rs`) fails if any service mounts a
  rootful socket.

### Rootless service

```sh
systemctl --user -M sandbox@ enable --now docker
```

Verify the host uses cgroup v2, the daemon reports
`Cgroup Version: 2` and `cgroupfs`/`systemd` driver, and its data root is not
`/var/lib/docker` (rootless uses `~/.local/share/docker`). The judge's daemon
identity check refuses the connection if any of these look wrong — there is no
skip flag.

## Images

Sandbox images are pulled before service time and pinned by digest in
`languages/languages.toml`. Sandboxes never pull at run time — a missing image
fails the run as infrastructure, it never reaches for the network.

To add or refresh an image:

```sh
docker pull python:3.13-slim
docker inspect --format '{{index .RepoDigests 0}}' python:3.13-slim
# -> python@sha256:...
docker pull python@sha256:<the digest>
```

Record the `repo@sha256:` pin in `languages/languages.toml`. The loader refuses
tags without a digest, so an unpinned entry can never reach production.

The weekly digest refresh is automated: Renovate (`renovate.json`) opens one
pull request per refresh that updates the `@sha256:` digest of each **enabled**
image, digest-only, no auto-merge. A human reviews and merges; `preflight`
must run on the VPS after every merge before user runs resume. The manual
procedure above stays the fallback for when a bot PR is not merged.

## Preflight on every deploy

After every deploy — and after any change to images, daemon, or limits — run
the judge's self-check against the sandbox daemon:

```sh
docker compose exec judge counterspell-judge preflight
# or, on the host with the judge binary:
SANDBOX_DOCKER_SOCKET=/run/user/$(id -u sandbox)/docker.sock \
  counterspell-judge preflight
```

Preflight writes into the sandbox container itself and compares the observed
kernel state (user, mounts, network, cgroup limits, seccomp) with the policy.
It exits non-zero on any failed check. Do not resume user runs until it
passes; record the output in `deploy/sandbox-verification.md`.

## Compose wiring

`docker-compose.yml` gives `judge` — and only `judge` — the sandbox socket:

```yaml
environment:
  SANDBOX_DOCKER_SOCKET: /sandbox/docker.sock
volumes:
  - ${SANDBOX_DOCKER_SOCKET_HOST:-/run/user/1000/docker.sock}:/sandbox/docker.sock
```

`SANDBOX_DOCKER_SOCKET_HOST` is set at deploy time to the sandbox user's
socket path. Mounting `/dev/null` there starts the judge without sandbox
access — every run reports infrastructure failure and nothing executes on the
host.

## Operational limits

Set the initial operational limit to:

```text
JUDGE_CONCURRENCY=2
```

Raise it only after VPS load and benchmark checks show stable timings. The
worker applies no network, a read-only root, bounded tmpfs scratch, a non-root
user, dropped capabilities, no-new-privileges, seccomp, CPU/memory/PID/
wall-time limits, bounded output, and cleanup/reaping on top of these host
limits — the two layers are complementary, never substitutes.
