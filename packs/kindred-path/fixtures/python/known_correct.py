def ember_path(vals, left, right, target):
    n = len(vals)
    if n == 0:
        return 0
    work = [(0, vals[0])]
    seen = set()
    while work:
        i, total = work.pop()
        if (i, total) in seen:
            continue
        seen.add((i, total))
        lo = left[i] if 0 <= left[i] < n else -1
        hi = right[i] if 0 <= right[i] < n else -1
        if lo == -1 and hi == -1:
            if total == target:
                return 1
            continue
        if lo != -1:
            work.append((lo, total + vals[lo]))
        if hi != -1:
            work.append((hi, total + vals[hi]))
    return 0
