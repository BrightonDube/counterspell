"""S07 — one allocation far past the cgroup memory limit. Python raises
MemoryError or the kernel kills the process; never accepted."""


def add(a: int, b: int) -> int:
    block = bytearray(1 << 33)  # 8 GiB in a single allocation
    return a + b + block[0]
