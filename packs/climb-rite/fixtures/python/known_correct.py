def climb_ways(steps):
    if steps <= 0:
        return 1
    a, b = 1, 1
    for _ in range(steps - 1):
        a, b = b, a + b
    return b
