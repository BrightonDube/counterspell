def tree_height(vals, left, right):
    n = len(vals)
    if n == 0:
        return 0
    best = 0
    seen = set()
    stack = [(0, 1)]
    while stack:
        i, d = stack.pop()
        if i in seen:
            continue
        seen.add(i)
        if d > best:
            best = d
        lo = left[i] if 0 <= left[i] < n else -1
        hi = right[i] if 0 <= right[i] < n else -1
        if lo != -1:
            stack.append((lo, d + 1))
        if hi != -1:
            stack.append((hi, d + 1))
    return best
