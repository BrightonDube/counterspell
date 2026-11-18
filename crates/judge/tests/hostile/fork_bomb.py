"""S08 — forks until the PID limit (64) refuses. The explosion stays inside the
container's own cgroup; the verdict is a runtime error or a time limit."""

import os


def add(a: int, b: int) -> int:
    while True:
        os.fork()
