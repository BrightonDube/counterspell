def hop_limit(n, links, ttl):
    if n == 0:
        return 0
    adj = [[] for _ in range(n)]
    for a, b in links:
        if 0 <= a < n and 0 <= b < n:
            adj[a].append(b)
            adj[b].append(a)
    dist = {0: 0}
    queue = [0]
    for u in queue:
        if dist[u] >= ttl:
            continue
        for v in adj[u]:
            if v not in dist:
                dist[v] = dist[u] + 1
                queue.append(v)
    return len(dist) - 1
