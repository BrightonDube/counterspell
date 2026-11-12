# Separate rootless sandbox daemon

The sandbox is a later judge feature and is deliberately not started by this
repository skeleton. It is a separate security boundary from the main Docker
Compose daemon.

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
- Only the trusted `counterspell-judge` worker may later receive access to the
  separate sandbox socket. The worker must not receive the main Docker daemon
  socket and must have no inbound port.

## Rootless service checks

Before enabling the judge on a target host, verify that the host uses cgroup v2
and that the `sandbox` user has cgroup delegation. Without delegation, CPU,
memory, and PID limits may not be effective. Record the check in deployment
notes before relying on those limits.

Runbooks for the target host should also verify that the rootless daemon uses
its own data root and socket, and that its permissions are restricted to the
`sandbox` user and the trusted judge deployment.

## Images and limits

Sandbox images are pulled before service time and pinned by digest in the
production language configuration. Sandboxes never pull images at run time.
The later judge implementation applies no network, a read-only root, bounded
scratch space, a non-root user, dropped capabilities, no-new-privileges,
seccomp, CPU/memory/PID/wall-time limits, bounded output, and cleanup/reaping.
This note does not implement code execution.

Set the initial operational limit to:

```text
JUDGE_CONCURRENCY=2
```

Raise it only after VPS load and benchmark checks show stable timings. Local
Compose for this skeleton does not require a sandbox daemon or sandbox socket.
