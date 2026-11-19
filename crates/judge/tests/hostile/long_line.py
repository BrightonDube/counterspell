"""S11 — one stdout line far longer than the budget. There is no newline to
make it parseable; the byte cap ends the call."""

import os


def add(a: int, b: int) -> int:
    os.write(1, b"y" * (8 * 1024 * 1024))
    return a + b
