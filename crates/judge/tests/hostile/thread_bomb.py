"""S08 — threads until the PID limit (counted per task) refuses. Same
containment as the fork bomb."""

import threading


def add(a: int, b: int) -> int:
    while True:
        threading.Thread(target=lambda: None).start()
