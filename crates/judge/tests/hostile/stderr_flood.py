"""S10 — an endless stderr flood that never answers. stderr is diagnostics, not
protocol output, so nothing here trips the output cap; the missed deadline —
not an output-limit kill — ends the call."""

import os


def add(a: int, b: int) -> int:
    while True:
        os.write(2, b"flood\n" * 700)
