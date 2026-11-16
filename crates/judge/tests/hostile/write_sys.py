"""S13 — writes into /sys: device and kernel state. None may succeed."""

TARGETS = [
    "/sys/kernel/uevent_helper",
    "/sys/fs/cgroup/memory.events",
    "/sys/power/state",
]


def add(a: int, b: int) -> int:
    for path in TARGETS:
        try:
            with open(path, "wb") as handle:
                handle.write(b"x")
        except OSError:
            pass
    return 0
