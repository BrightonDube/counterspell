# S15: closes stdout under the driver. The answer can never leave the pipe; the call
# must surface as a driver exit, not a hang or a verdict.
import os
import sys


def add(a, b):
    try:
        sys.stdout.close()
    finally:
        os.close(1)
    return a + b
