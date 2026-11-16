"""S06 — the loop runs while the module loads, before the driver can answer
a single call. The process stays alive so the deadline, not a driver exit,
decides the test."""

while True:
    pass


def add(a: int, b: int) -> int:
    return a + b
