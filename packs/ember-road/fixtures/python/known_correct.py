import heapq


def road_toll(edges, weights, n):
    if n <= 0:
        return -1
    adj = [[] for _ in range(n)]
    for k in range(min(len(edges), len(weights))):
        a, b = edges[k]
        c = weights[k]
        if 0 <= a < n and 0 <= b < n:
            adj[a].append((b, c))
            adj[b].append((a, c))
    dist = [None] * n
    dist[0] = 0
    pq = [(0, 0)]
    while pq:
        du, u = heapq.heappop(pq)
        if dist[u] is not None and du > dist[u]:
            continue
        for v, c in adj[u]:
            nd = du + c
            if dist[v] is None or nd < dist[v]:
                dist[v] = nd
                heapq.heappush(pq, (nd, v))
    return dist[n - 1] if dist[n - 1] is not None else -1
