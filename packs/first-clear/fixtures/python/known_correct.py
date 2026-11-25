def first_clear(heights, mark):
    lo, hi = 0, len(heights)
    while lo < hi:
        mid = (lo + hi) // 2
        if heights[mid] < mark:
            lo = mid + 1
        else:
            hi = mid
    return lo