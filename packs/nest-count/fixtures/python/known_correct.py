def nest_pairs(spans):
    n = 0
    size = len(spans)
    for i in range(size):
        for j in range(i + 1, size):
            a, b = spans[i]
            c, d = spans[j]
            if (a < c and d < b) or (c < a and b < d):
                n += 1
    return n
