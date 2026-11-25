def find_ember(embers, target):
    lo, hi = 0, len(embers)
    while lo < hi:
        mid = (lo + hi) // 2
        if embers[mid] < target:
            lo = mid + 1
        else:
            hi = mid
    return lo if lo < len(embers) and embers[lo] == target else -1