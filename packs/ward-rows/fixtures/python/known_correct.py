def row_ways(length):
    if length <= 0:
        return 1
    d = [0] * (length + 1)
    d[0] = 1
    for i in range(1, length + 1):
        for p in (1, 2, 3):
            if i - p >= 0:
                d[i] += d[i - p]
    return d[length]
