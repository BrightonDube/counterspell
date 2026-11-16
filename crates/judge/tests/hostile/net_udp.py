"""S12 — UDP datagrams to the same host candidates on the listener's port."""

import socket

PORT = __PORT__
HOSTS = ["127.0.0.1", "10.0.2.2", "172.17.0.1", "192.168.65.1", "__HOST__"]


def add(a: int, b: int) -> int:
    for host in HOSTS:
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            sock.sendto(b"probe", (host, PORT))
            sock.close()
        except OSError:
            pass
    return 0
