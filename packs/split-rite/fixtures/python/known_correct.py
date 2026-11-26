def split_rite(n, links, down):
    dead = set(down)
    adj = [[] for _ in range(n)]
    for i, (a, b) in enumerate(links):
        if i in dead:
            continue
        if 0 <= a < n and 0 <= b < n:
            adj[a].append(b)
            adj[b].append(a)
    seen = set()
    quorum = 0
    for s in range(n):
        if s in seen:
            continue
        comp, queue = set(), [s]
        seen.add(s)
        for u in queue:
            comp.add(u)
            for v in adj[u]:
                if v not in seen:
                    seen.add(v)
                    queue.append(v)
        if len(comp) * 2 > n:
            quorum += 1
    return quorum
