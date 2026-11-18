"""S13 — one file far bigger than the tmpfs and the fsize ulimit."""


def add(a: int, b: int) -> int:
    try:
        with open("/box/huge", "wb") as handle:
            handle.write(b"x" * (1 << 30))
    except (OSError, ValueError):
        pass
    return 0
