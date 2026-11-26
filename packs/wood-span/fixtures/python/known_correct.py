def span_toll(edges, weights, n):
    if n <= 0:
        return 0
    parent = list(range(n))

    def find(x):
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x

    arcs = []
    for k in range(min(len(edges), len(weights))):
        a, b = edges[k]
        c = weights[k]
        if 0 <= a < n and 0 <= b < n and a != b:
            arcs.append((c, a, b))
    arcs.sort()
    total = 0
    used = 0
    for c, a, b in arcs:
        ra, rb = find(a), find(b)
        if ra != rb:
            parent[ra] = rb
            total += c
            used += 1
    return total if used == n - 1 else -1
