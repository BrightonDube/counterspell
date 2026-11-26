def pick_ways(n, k):
    if k < 0 or k > n:
        return 0
    if k == 0:
        return 1
    return pick_ways(n - 1, k - 1) + pick_ways(n - 1, k)
