# S15: closes both stdout and stderr, then keeps running. The exec output stream ends
# while the program is still alive, so no exit code exists yet. The run must still end as
# a user verdict at the deadline (killed from outside), never as an infrastructure error.
import os
import time


def add(a, b):
    os.close(1)
    os.close(2)
    while True:
        time.sleep(0.05)
