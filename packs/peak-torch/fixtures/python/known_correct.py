def peak_torch(heights):
    lo, hi = 0, len(heights) - 1
    while lo < hi:
        mid = (lo + hi) // 2
        if heights[mid] < heights[mid + 1]:
            lo = mid + 1
        else:
            hi = mid
    return lo
