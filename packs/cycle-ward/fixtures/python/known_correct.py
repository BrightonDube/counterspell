def cycle_check(edges, n):
    parent = list(range(n))

    def find(x):
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x

    for a, b in edges:
        if not (0 <= a < n and 0 <= b < n):
            continue
        if a == b:
            return 1
        ra, rb = find(a), find(b)
        if ra == rb:
            return 1
        parent[ra] = rb
    return 0
