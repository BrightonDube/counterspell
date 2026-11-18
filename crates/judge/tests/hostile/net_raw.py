"""S12 — a raw socket needs CAP_NET_RAW, which the policy drops. Opening one
must fail; even if it opened there is no route out."""

import socket


def add(a: int, b: int) -> int:
    try:
        sock = socket.socket(socket.AF_INET, socket.SOCK_RAW, socket.IPPROTO_ICMP)
        sock.sendto(b"\x08\x00\xf7\xff\x00\x00\x00\x00", ("10.0.2.2", 0))
        sock.close()
    except OSError:
        pass
    return 0
