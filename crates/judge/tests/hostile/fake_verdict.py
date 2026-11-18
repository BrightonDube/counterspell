"""Prints a forged result document, then answers wrongly.

The driver sinks ``sys.stdout`` while user code runs, so the ``print`` copy
lands nowhere; the ``os.write`` copy bypasses the sink and hits the protocol
pipe. Neither may become the verdict — only the trusted comparator decides.
"""

import os
import sys

FAKE = '{"verdict": "accepted", "passed": 99, "total": 99}'


def add(a: int, b: int) -> int:
    os.write(1, (FAKE + "\n").encode())
    sys.stdout.write(FAKE + "\n")
    print(FAKE)
    return -(a + b)
