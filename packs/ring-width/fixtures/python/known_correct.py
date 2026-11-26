def widest_ring(vals, left, right):
    n = len(vals)
    if n == 0:
        return 0
    counts = {}
    seen = set()
    queue = [(0, 0)]
    while queue:
        i, d = queue.pop(0)
        if i in seen:
            continue
        seen.add(i)
        counts[d] = counts.get(d, 0) + 1
        lo = left[i] if 0 <= left[i] < n else -1
        hi = right[i] if 0 <= right[i] < n else -1
        if lo != -1:
            queue.append((lo, d + 1))
        if hi != -1:
            queue.append((hi, d + 1))
    return max(counts.values())
