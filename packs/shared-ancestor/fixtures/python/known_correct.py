def first_ancestor(vals, left, right, a, b):
    n = len(vals)
    if n == 0 or not (0 <= a < n) or not (0 <= b < n):
        return -1
    parent = {}
    seen = set()
    stack = [0]
    while stack:
        i = stack.pop()
        if i in seen:
            continue
        seen.add(i)
        for arr in (left, right):
            k = arr[i] if 0 <= arr[i] < n else -1
            if k != -1 and k not in parent:
                parent[k] = i
                stack.append(k)
    if a not in seen or b not in seen:
        return -1
    anc = set()
    x = a
    while x != -1:
        anc.add(x)
        x = parent.get(x, -1)
    x = b
    while x != -1:
        if x in anc:
            return x
        x = parent.get(x, -1)
    return -1
