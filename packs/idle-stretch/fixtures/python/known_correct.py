def idle_stretch(spans, t):
    cov = [False] * (t + 1)
    for a, b in spans:
        for i in range(max(a, 0), min(b, t) + 1):
            cov[i] = True
    best = 0
    run = 0
    for i in range(t + 1):
        run = 0 if cov[i] else run + 1
        if run > best:
            best = run
    return best
