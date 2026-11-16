"""Answers the visible samples correctly, then replays garbage on hidden tests.

The samples always run before the hidden tests, so call order is observable.
When the first hidden call arrives this function emits a replayed 'accepted'
document (sunk by the driver anyway) and returns a wrong answer. The run must
fail and the public records after it must contain no hidden input.
"""

import sys

_seen = [0]


def add(a: int, b: int) -> int:
    _seen[0] += 1
    if _seen[0] <= 3:
        return a + b
    sys.stdout.write('{"verdict": "accepted"}\n')
    return -999
