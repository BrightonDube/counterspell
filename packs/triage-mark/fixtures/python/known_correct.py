def next_mark(prios):
    best = -1
    for i, p in enumerate(prios):
        if best < 0 or p < prios[best]:
            best = i
    return best
