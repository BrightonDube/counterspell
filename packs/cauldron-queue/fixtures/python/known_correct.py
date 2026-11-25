from collections import deque


def servings(ops):
    line = deque()
    served = 0
    for op in ops:
        if op == 0:
            if line:
                line.popleft()
                served += 1
        else:
            line.append(op)
    return served
