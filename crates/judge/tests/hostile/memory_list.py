"""S07 — steady growth until the cgroup memory limit stops the program. The
kernel kills the process (oom_kill > 0) or raises MemoryError; never accepted."""


def add(a: int, b: int) -> int:
    blocks = []
    while True:
        blocks.append(bytearray(1 << 20))
