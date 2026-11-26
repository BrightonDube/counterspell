def route_count(n, links):
    if n == 0:
        return 0
    adj = [[] for _ in range(n)]
    for a, b in links:
        if 0 <= a < n and 0 <= b < n:
            adj[a].append(b)
            adj[b].append(a)
    seen, queue = {0}, [0]
    for u in queue:
        for v in adj[u]:
            if v not in seen:
                seen.add(v)
                queue.append(v)
    return len(seen) - 1
