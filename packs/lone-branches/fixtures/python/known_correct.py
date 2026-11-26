def single_children(vals, left, right):
    n = len(vals)
    count = 0
    seen = set()
    stack = [0] if n else []
    while stack:
        i = stack.pop()
        if i in seen:
            continue
        seen.add(i)
        lo = left[i] if 0 <= left[i] < n else -1
        hi = right[i] if 0 <= right[i] < n else -1
        if (lo == -1) != (hi == -1):
            count += 1
        if lo != -1:
            stack.append(lo)
        if hi != -1:
            stack.append(hi)
    return count
