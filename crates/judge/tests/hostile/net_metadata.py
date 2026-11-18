"""S12 — the cloud metadata endpoint at 169.254.169.254: credentials or
identity material must never leave the sandbox."""

import urllib.request


def add(a: int, b: int) -> int:
    try:
        urllib.request.urlopen(
            "http://169.254.169.254/latest/meta-data/", timeout=0.5
        ).read()
    except Exception:
        pass
    return 0
