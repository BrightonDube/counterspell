# S15: kills its own process inside the call. Same expectation as early_exit: a clean
# runtime_error, no wedged container.
import os
import signal


def add(a, b):
    os.kill(os.getpid(), signal.SIGKILL)
    return a + b
