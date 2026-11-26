RING = -10**15


def ebb_toll(edges, weights, n):
    if n <= 0:
        return -1
    arcs = []
    for k in range(min(len(edges), len(weights))):
        a, b = edges[k]
        c = weights[k]
        if 0 <= a < n and 0 <= b < n:
            arcs.append((a, b, c))
    inf = float("inf")
    dist = [inf] * n
    dist[0] = 0
    for _ in range(n - 1):
        for a, b, c in arcs:
            if dist[a] < inf and dist[a] + c < dist[b]:
                dist[b] = dist[a] + c
    marked = [False] * n
    for _ in range(n):
        for a, b, c in arcs:
            if (dist[a] < inf and dist[a] + c < dist[b]) or marked[a]:
                marked[b] = True
    if marked[n - 1]:
        return RING
    return int(dist[n - 1]) if dist[n - 1] < inf else -1
