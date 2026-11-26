import heapq


def ferry_load(edges, weights, n):
    if n <= 1:
        return 0 if n == 1 else -1
    adj = [[] for _ in range(n)]
    for k in range(min(len(edges), len(weights))):
        a, b = edges[k]
        c = weights[k]
        if 0 <= a < n and 0 <= b < n:
            adj[a].append((b, c))
            adj[b].append((a, c))
    best = [-1] * n
    best[0] = 10**30
    pq = [(-best[0], 0)]
    while pq:
        neg_cap, u = heapq.heappop(pq)
        cap = -neg_cap
        if cap < best[u]:
            continue
        for v, c in adj[u]:
            cand = min(cap, c)
            if cand > best[v]:
                best[v] = cand
                heapq.heappush(pq, (-cand, v))
    return best[n - 1] if best[n - 1] >= 0 else -1
