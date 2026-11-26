def tree_diameter(vals, left, right):
    n = len(vals)
    if n == 0:
        return 0
    heights = [-1] * n
    best = 0
    seen = set()
    stack = [(0, False)]
    while stack:
        i, done = stack.pop()
        if i in seen:
            continue
        if not done:
            stack.append((i, True))
            for arr in (left, right):
                k = arr[i] if 0 <= arr[i] < n else -1
                if k != -1:
                    stack.append((k, False))
            continue
        seen.add(i)
        lh = rh = 0
        lo = left[i] if 0 <= left[i] < n else -1
        hi = right[i] if 0 <= right[i] < n else -1
        if lo != -1 and heights[lo] >= 0:
            lh = heights[lo] + 1
        if hi != -1 and heights[hi] >= 0:
            rh = heights[hi] + 1
        if lh + rh > best:
            best = lh + rh
        heights[i] = lh if lh > rh else rh
    return best
