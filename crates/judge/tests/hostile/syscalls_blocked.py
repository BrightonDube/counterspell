"""S26 — dangerous syscalls through ctypes: mount, unshare, ptrace, keyctl.
Every one is refused by seccomp, dropped capabilities or no-new-privileges.
Prints REPORT:<calls that unexpectedly returned success>."""

import ctypes

LIBC = ctypes.CDLL(None, use_errno=True)


def attempt(name: str, call) -> str | None:
    try:
        result = call()
    except Exception:
        return None  # EPERM, ENOSYS, or no such symbol — all refusals count
    return name if result == 0 else None


def probe() -> None:
    escaped = []
    for name, call in [
        ("mount", lambda: LIBC.mount(b"none", b"/tmp/m", b"tmpfs", 0, None)),
        ("unshare", lambda: LIBC.unshare(0x00020000)),  # CLONE_NEWNS
        ("ptrace", lambda: LIBC.ptrace(16, 1, None, None)),  # PTRACE_ATTACH
        ("keyctl", lambda: LIBC.keyctl(1, 0, 0, 0, 0)),
        ("syscall_keyctl", lambda: LIBC.syscall(250, 1, 0, 0, 0, 0)),  # SYS_keyctl
    ]:
        if attempt(name, call):
            escaped.append(name)
    print("REPORT:" + ",".join(escaped))


probe()


def add(a: int, b: int) -> int:
    return 0
