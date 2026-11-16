"""S09 — a text flood on the program's own stdout stream. `sys.__stdout__` is
the real stream (the driver's sink only replaced `sys.stdout`), so the bytes
reach the protocol pipe. They carry no newline, so the byte cap — not a parse
failure — ends the call."""

import sys


def add(a: int, b: int) -> int:
    while True:
        sys.__stdout__.write("x" * 4096)
        sys.__stdout__.flush()
