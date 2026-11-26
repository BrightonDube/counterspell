def reach_count(edges, n):
    if n <= 0:
        return 0
    adj = [[] for _ in range(n)]
    for a, b in edges:
        if 0 <= a < n and 0 <= b < n:
            adj[a].append(b)
            adj[b].append(a)
    seen = [False] * n
    seen[0] = True
    stack = [0]
    count = 0
    while stack:
        x = stack.pop()
        count += 1
        for y in adj[x]:
            if not seen[y]:
                seen[y] = True
                stack.append(y)
    return count
