"""S05 — an answer that never arrives. The call deadline fires, the worker
kills the container from outside, and the test is a `time_limit`."""


def add(a: int, b: int) -> int:
    while True:
        pass
