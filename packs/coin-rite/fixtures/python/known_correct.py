def coin_rite(coins, target):
    d = [None] * (target + 1)
    d[0] = 0
    for i in range(1, target + 1):
        best = None
        for c in coins:
            if i - c >= 0 and d[i - c] is not None:
                v = d[i - c] + 1
                best = v if best is None else min(best, v)
        d[i] = best
    return d[target] if d[target] is not None else -1
