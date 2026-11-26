import heapq


def hop_toll(edges, weights, n, k):
    if n <= 0 or k < 0:
        return -1
    adj = [[] for _ in range(n)]
    for i in range(min(len(edges), len(weights))):
        a, b = edges[i]
        c = weights[i]
        if 0 <= a < n and 0 <= b < n:
            adj[a].append((b, c))
            adj[b].append((a, c))
    dist = [[None] * (k + 1) for _ in range(n)]
    dist[0][0] = 0
    pq = [(0, 0, 0)]
    while pq:
        du, u, h = heapq.heappop(pq)
        if dist[u][h] is not None and du > dist[u][h]:
            continue
        for v, c in adj[u]:
            if h + 1 <= k:
                nd = du + c
                if dist[v][h + 1] is None or nd < dist[v][h + 1]:
                    dist[v][h + 1] = nd
                    heapq.heappush(pq, (nd, v, h + 1))
    best = min(d for d in dist[n - 1] if d is not None) if any(d is not None for d in dist[n - 1]) else None
    return best if best is not None else -1
