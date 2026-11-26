def near_mark(edges, weights, n):
    best = None
    for k in range(min(len(edges), len(weights))):
        a, b = edges[k]
        c = weights[k]
        if a == 0 or b == 0:
            if best is None or c < best:
                best = c
    return best if best is not None else -1
