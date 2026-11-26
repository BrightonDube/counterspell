def split_diff(marks):
    total = sum(marks)
    half = total // 2
    d = [False] * (half + 1)
    d[0] = True
    for m in marks:
        for i in range(half, m - 1, -1):
            if d[i - m]:
                d[i] = True
    best = max(i for i in range(half + 1) if d[i])
    return total - 2 * best
