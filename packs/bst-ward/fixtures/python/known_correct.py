def bst_check(vals, left, right):
    n = len(vals)
    if n == 0:
        return 1
    neg, pos = float("-inf"), float("inf")
    work = [(0, neg, pos)]
    seen = set()
    while work:
        i, lo, hi = work.pop()
        if not lo < vals[i] < hi:
            return 0
        key = (i, lo, hi)
        if key in seen:
            continue
        seen.add(key)
        l = left[i] if 0 <= left[i] < n else -1
        r = right[i] if 0 <= right[i] < n else -1
        if l != -1:
            work.append((l, lo, vals[i]))
        if r != -1:
            work.append((r, vals[i], hi))
    return 1
