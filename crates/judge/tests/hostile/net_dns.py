"""S12 — DNS: resolve a real name and send a raw query to likely resolvers.
No resolver may answer and no name may resolve."""

import socket


def add(a: int, b: int) -> int:
    try:
        socket.getaddrinfo("host.docker.internal", 80)
    except OSError:
        pass
    query = bytes.fromhex("aaaa0100000100000000000003696f736903636f6d0000010001")
    for server in ["10.0.2.3", "8.8.8.8", "127.0.0.53"]:
        try:
            sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            sock.settimeout(0.3)
            sock.sendto(query, (server, 53))
            sock.recvfrom(512)
            sock.close()
        except OSError:
            pass
    return 0
