# T35 sbx_logs: writes a canary to stderr and then raises, so the exception
# class/message carry the canary back through the driver's answer line. The
# verdict is a crashed call — the canary must never reach worker logs.
import os

CANARY = "CANARY-EXC-4d91"


def add(a, b):
    os.write(2, (CANARY + "\n").encode())
    raise Exception("the failure says " + CANARY)
