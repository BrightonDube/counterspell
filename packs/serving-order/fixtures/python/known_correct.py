from collections import deque


def serve_order(ops):
    line = deque()
    served = []
    for op in ops:
        if op == 0:
            if line:
                served.append(line.popleft())
        else:
            line.append(op)
    return served
