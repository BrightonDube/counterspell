def order_count(edges, n):
    adj = [[] for _ in range(n)]
    indeg = [0] * n
    self_order = False
    for a, b in edges:
        if not (0 <= a < n and 0 <= b < n):
            continue
        if a == b:
            self_order = True
            continue
        if b not in adj[a]:
            adj[a].append(b)
            indeg[b] += 1
    if self_order:
        return 0

    total = 0

    def go(k):
        nonlocal total
        if k == n:
            total += 1
            return
        for v in range(n):
            if indeg[v] == 0:
                indeg[v] = -1
                for w in adj[v]:
                    indeg[w] -= 1
                go(k + 1)
                for w in adj[v]:
                    indeg[w] += 1
                indeg[v] = 0

    go(0)
    return total
