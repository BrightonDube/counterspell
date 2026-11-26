def pack_worth(items, cap):
    d = [0] * (cap + 1)
    for w, v in items:
        for i in range(cap, w - 1, -1):
            d[i] = max(d[i], d[i - w] + v)
    return d[cap]
