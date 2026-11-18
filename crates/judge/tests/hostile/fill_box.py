"""S13 — fills /box, the only writable scratch, until the tmpfs size cap
refuses. The fill counts against the memory limit too."""


def add(a: int, b: int) -> int:
    for index in range(200):
        try:
            with open(f"/box/fill{index}", "wb") as handle:
                handle.write(b"x" * (1 << 20))
        except OSError:
            pass
    return 0
