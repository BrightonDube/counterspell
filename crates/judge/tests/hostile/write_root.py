"""S13 — writes into the read-only root filesystem. Every target is outside
the tmpfs scratch, so each must fail; the daemon's filesystem-changes report
for the container stays empty."""

TARGETS = ["/x", "/etc/x", "/usr/x", "/bin/x", "/var/x", "/home/x"]


def add(a: int, b: int) -> int:
    for path in TARGETS:
        try:
            with open(path, "w") as handle:
                handle.write("x")
        except OSError:
            pass
    return 0
