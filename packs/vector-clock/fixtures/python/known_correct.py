def clock_cmp(a, b):
    less = any(x < y for x, y in zip(a, b))
    more = any(x > y for x, y in zip(a, b))
    if less and more:
        return 2
    if less:
        return -1
    if more:
        return 1
    return 0
