def span_cover(spans, t):
    s = sorted(spans)
    front = 0
    i = 0
    n = 0
    while front <= t:
        best = -1
        while i < len(s) and s[i][0] <= front:
            best = max(best, s[i][1])
            i += 1
        if best < front:
            return -1
        n += 1
        if best >= t:
            return n
        front = best + 1
    return n
