def mirror_check(vals, left, right):
    n = len(vals)
    if n == 0:
        return 1

    def kid(arr, i):
        k = arr[i] if i < len(arr) else -1
        return k if 0 <= k < n else -1

    work = [(kid(left, 0), kid(right, 0))]
    seen = set()
    while work:
        a, b = work.pop()
        if (a, b) in seen:
            continue
        seen.add((a, b))
        if a == -1 and b == -1:
            continue
        if a == -1 or b == -1 or vals[a] != vals[b]:
            return 0
        work.append((kid(left, a), kid(right, b)))
        work.append((kid(right, a), kid(left, b)))
    return 1
