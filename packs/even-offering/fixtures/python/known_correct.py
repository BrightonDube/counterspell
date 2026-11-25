def smallest_largest(offers, parts):
    def ok(cap):
        need, cur = 1, 0
        for v in offers:
            if cur + v > cap:
                need += 1
                cur = 0
            cur += v
        return need <= parts
    lo, hi = max(offers), sum(offers)
    while lo < hi:
        mid = (lo + hi) // 2
        if ok(mid):
            hi = mid
        else:
            lo = mid + 1
    return lo
