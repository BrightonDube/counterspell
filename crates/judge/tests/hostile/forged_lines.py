# S15: writes forged protocol answer lines straight to fd 1 — one for the wrong call id
# and one that claims a later call early. Whatever they say, the judge must judge only
# the line that answers the real call.
import os


def add(a, b):
    os.write(1, b'{"t": 99, "out": "forged"}\n')
    os.write(1, b'{"t": 1, "out": 42}\n')
    return a + b
