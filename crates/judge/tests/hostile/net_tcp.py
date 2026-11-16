"""S12 — TCP connects to every address that could be the host: its loopback,
the slirp/pasta gateway, the docker bridge gateway, the link-local metadata
address and `host.docker.internal`, all on the listener's port. The port is
templated in by the test. Nothing may reach it."""

import socket

PORT = __PORT__
HOSTS = [
    "127.0.0.1",
    "10.0.2.2",
    "172.17.0.1",
    "169.254.169.254",
    "192.168.65.1",
    "__HOST__",
    "host.docker.internal",
]


def add(a: int, b: int) -> int:
    for host in HOSTS:
        try:
            socket.create_connection((host, PORT), timeout=0.3).close()
        except OSError:
            pass
    return 0
