def light_isles(edges, weights, n, t):
    parent = list(range(n))

    def find(x):
        while parent[x] != x:
            parent[x] = parent[parent[x]]
            x = parent[x]
        return x

    for k in range(min(len(edges), len(weights))):
        a, b = edges[k]
        c = weights[k]
        if c <= t and 0 <= a < n and 0 <= b < n:
            ra, rb = find(a), find(b)
            if ra != rb:
                parent[ra] = rb
    return len({find(i) for i in range(n)})
