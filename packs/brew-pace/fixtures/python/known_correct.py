def min_pace(brews, hours):
    lo, hi = 1, max(brews)
    while lo < hi:
        mid = (lo + hi) // 2
        need = sum((t + mid - 1) // mid for t in brews)
        if need <= hours:
            hi = mid
        else:
            lo = mid + 1
    return lo
