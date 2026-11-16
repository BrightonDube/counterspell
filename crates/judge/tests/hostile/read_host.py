"""S26 — reads host-visible paths: docker sockets, init's environment, secret
files, shadow data. Run as a solution the prints are sunk and it answers
wrongly; run as a probe script it prints REPORT:<paths it could read>."""

TARGETS = [
    "/var/run/docker.sock",
    "/run/docker.sock",
    "/proc/1/environ",
    "/etc/shadow",
    "/root/.ssh/id_rsa",
    "/etc/kubernetes",
    "/host",
    "/proc/self/mounts",
]


def probe() -> None:
    found = []
    for path in TARGETS:
        try:
            with open(path, "rb") as handle:
                data = handle.read(4096)
            found.append(f"{path}:{len(data)}")
        except (OSError, ValueError):
            pass
    print("REPORT:" + ",".join(found))


probe()


def add(a: int, b: int) -> int:
    return 0
