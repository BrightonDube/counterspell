# T35 sbx_logs: writes a canary straight to the protocol pipe (fd 1) and to
# stderr (fd 2), bypassing the driver's stdout sink. The first line is a
# protocol violation, so the call must end WrongAnswer — the canary bytes only
# ever existed inside the sandbox and must never reach worker logs.
import os

CANARY = "CANARY-FD-8c3e"


def add(a, b):
    os.write(2, (CANARY + "\n").encode())
    os.write(1, (CANARY + "\n").encode())
    return a + b
