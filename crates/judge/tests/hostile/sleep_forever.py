"""S06 — a sleep no signal from inside can shorten. Only the worker's deadline
ends the call."""

import time


def add(a: int, b: int) -> int:
    time.sleep(1e9)
    return a + b
