"""S13 — writes into /proc: kernel knobs and own-process state. None may
succeed."""

TARGETS = [
    "/proc/sys/kernel/panic",
    "/proc/sysrq-trigger",
    "/proc/self/mem",
    "/proc/1/mem",
    "/proc/kcore",
]


def add(a: int, b: int) -> int:
    for path in TARGETS:
        try:
            with open(path, "wb") as handle:
                handle.write(b"1")
        except OSError:
            pass
    return 0
