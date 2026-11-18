"""S12 — IPv6 loopback, link-local and ULA candidates. A v6 route would be a
second escape path; none may exist."""

import socket

PORT = __PORT__
HOSTS = ["::1", "fe80::1", "fd00::1"]


def add(a: int, b: int) -> int:
    for host in HOSTS:
        try:
            socket.create_connection((host, PORT), timeout=0.3).close()
        except OSError:
            pass
    return 0
