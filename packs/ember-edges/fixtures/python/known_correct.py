def ember_edges(embers, target):
    def lower(mark):
        lo, hi = 0, len(embers)
        while lo < hi:
            mid = (lo + hi) // 2
            if embers[mid] < mark:
                lo = mid + 1
            else:
                hi = mid
        return lo
    first = lower(target)
    if first == len(embers) or embers[first] != target:
        return [-1, -1]
    return [first, lower(target + 1) - 1]
