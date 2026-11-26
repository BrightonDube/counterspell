def bridge_count(edges, n):
    adj = [[] for _ in range(n)]
    arcs = []
    for a, b in edges:
        if 0 <= a < n and 0 <= b < n and a != b:
            idx = len(arcs)
            arcs.append((a, b))
            adj[a].append((b, idx))
            adj[b].append((a, idx))
    disc = [-1] * n
    low = [0] * n
    stamps = [0]
    bridges = [0]

    def dfs(u, entry_arc):
        disc[u] = low[u] = stamps[0]
        stamps[0] += 1
        for v, arc in adj[u]:
            if arc == entry_arc:
                continue
            if disc[v] == -1:
                dfs(v, arc)
                low[u] = min(low[u], low[v])
                if low[v] > disc[u]:
                    bridges[0] += 1
            else:
                low[u] = min(low[u], disc[v])

    for s in range(n):
        if disc[s] == -1:
            dfs(s, -1)
    return bridges[0]
