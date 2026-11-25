def count_below(marks, toll):
    lo, hi = 0, len(marks)
    while lo < hi:
        mid = (lo + hi) // 2
        if marks[mid] < toll:
            lo = mid + 1
        else:
            hi = mid
    return lo