def highest_floor(steps, reach):
    lo, hi = 0, len(steps)
    while lo < hi:
        mid = (lo + hi) // 2
        if steps[mid] <= reach:
            lo = mid + 1
        else:
            hi = mid
    return steps[lo - 1] if lo > 0 else -1