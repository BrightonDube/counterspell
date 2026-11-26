def pick_count(spans):
    n = 0
    end = -1
    for a, b in sorted(spans, key=lambda s: s[1]):
        if a > end:
            n += 1
            end = b
    return n
