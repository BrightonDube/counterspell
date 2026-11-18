"""S09 — the same flood through raw `os.write`, no newline anywhere. The
worker cuts it at the stdout budget."""

import os


def add(a: int, b: int) -> int:
    while True:
        os.write(1, b"x" * 4096)
