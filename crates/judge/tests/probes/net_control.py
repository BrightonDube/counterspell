"""Trusted control for S12 (never hostile, never part of a verdict): run inside
a deliberately network-enabled container, it tries the candidate host
addresses on the host listener's ports and reports which were reachable. The
TCP and UDP ports arrive as argv[1] and argv[2]; the gateway is read from /proc/net/route. If nothing
is reachable, the S12 test reports "control inconclusive" — a silent pass
would mean the listener could not have seen a connection anyway.
"""

import socket
import struct
import sys


def gateways() -> list:
    found = []
    try:
        for line in open("/proc/net/route").read().splitlines()[1:]:
            fields = line.split()
            if len(fields) > 2 and fields[1] == "00000000":
                found.append(socket.inet_ntoa(struct.pack("<L", int(fields[2], 16))))
    except OSError:
        pass
    return found


def main() -> None:
    tcp_port = int(sys.argv[1])
    udp_port = int(sys.argv[2])
    hosts = ["10.0.2.2", "172.17.0.1", "host.docker.internal", "192.168.65.1"]
    hosts.extend(sys.argv[3:])  # the host's LAN addresses, detected on the host side
    for gateway in gateways():
        if gateway not in hosts:
            hosts.append(gateway)
    reached = []
    for host in hosts:
        try:
            socket.create_connection((host, tcp_port), timeout=2.0).close()
            reached.append(host)
        except OSError:
            pass
        try:
            udp = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
            udp.sendto(b"probe", (host, udp_port))
            udp.close()
        except OSError:
            pass
    print("REACHABLE:" + ",".join(reached) if reached else "UNREACHABLE")


main()
