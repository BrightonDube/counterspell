def watch_points(spans):
    s = sorted(((a, b) for a, b in spans if a <= b), key=lambda s: s[1])
    pts = 0
    i = 0
    while i < len(s):
        p = s[i][1]
        pts += 1
        while i < len(s) and s[i][0] <= p:
            i += 1
    return pts
