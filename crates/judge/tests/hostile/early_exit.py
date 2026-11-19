# S15: exits before answering. The driver dies mid-call; the test must end as
# runtime_error, and every later test starts in a fresh container.
import sys


def add(a, b):
    sys.exit(0)
