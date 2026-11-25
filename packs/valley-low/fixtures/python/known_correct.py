def lowest_mark(rotated):
    lo, hi = 0, len(rotated) - 1
    while lo < hi:
        mid = (lo + hi) // 2
        if rotated[mid] > rotated[hi]:
            lo = mid + 1
        else:
            hi = mid
    return rotated[lo]
