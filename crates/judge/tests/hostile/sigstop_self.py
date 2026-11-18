"""S06 — SIGSTOP makes the process unanswerable without killing it. SIGKILL
still works on a stopped process, so the deadline path ends the call."""

import os
import signal


def add(a: int, b: int) -> int:
    os.kill(os.getpid(), signal.SIGSTOP)
    return a + b
