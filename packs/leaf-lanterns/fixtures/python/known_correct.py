def count_leaves(vals, left, right):
    n = len(vals)
    seen = set()
    stack = [0] if n else []
    leaves = 0
    while stack:
        i = stack.pop()
        if i in seen:
            continue
        seen.add(i)
        lo = left[i] if 0 <= left[i] < n else -1
        hi = right[i] if 0 <= right[i] < n else -1
        if lo == -1 and hi == -1:
            leaves += 1
        else:
            if lo != -1:
                stack.append(lo)
            if hi != -1:
                stack.append(hi)
    return leaves
