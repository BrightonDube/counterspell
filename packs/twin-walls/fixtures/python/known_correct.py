def twin_walls(edges, n):
    adj = [[] for _ in range(n)]
    for a, b in edges:
        if 0 <= a < n and 0 <= b < n:
            adj[a].append(b)
            adj[b].append(a)
    colour = [None] * n
    for s in range(n):
        if colour[s] is not None:
            continue
        colour[s] = 0
        stack = [s]
        while stack:
            x = stack.pop()
            for y in adj[x]:
                if colour[y] is None:
                    colour[y] = 1 - colour[x]
                    stack.append(y)
                elif colour[y] == colour[x]:
                    return 0
    return 1
