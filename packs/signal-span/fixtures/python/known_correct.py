def span_max(edges, n):
    if n <= 0:
        return 0
    adj = [[] for _ in range(n)]
    for a, b in edges:
        if 0 <= a < n and 0 <= b < n:
            adj[a].append(b)
            adj[b].append(a)
    dist = [-1] * n
    dist[0] = 0
    queue = [0]
    for x in queue:
        for y in adj[x]:
            if dist[y] < 0:
                dist[y] = dist[x] + 1
                queue.append(y)
    return max(dist)
