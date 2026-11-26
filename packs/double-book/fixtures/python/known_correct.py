def room_count(spans):
    evs = []
    for a, b in spans:
        evs.append((a, 1))
        evs.append((b, -1))
    evs.sort(key=lambda e: (e[0], -e[1]))
    cur = 0
    best = 0
    for _, d in evs:
        cur += d
        if cur > best:
            best = cur
    return best
