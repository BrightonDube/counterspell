def relay_count(spans, gap):
    n = 0
    end = -(10**9)
    for a, b in sorted(spans, key=lambda s: s[1]):
        if a >= end + gap:
            n += 1
            end = b
    return n
