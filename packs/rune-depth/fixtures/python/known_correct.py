def rune_depth(edges, n):
    adj = [[] for _ in range(n)]
    self_order = False
    for a, b in edges:
        if not (0 <= a < n and 0 <= b < n):
            continue
        if a == b:
            self_order = True
            continue
        if b not in adj[a]:
            adj[a].append(b)
    if self_order:
        return 0
    state = [0] * n
    memo = [0] * n

    def dfs(u):
        state[u] = 1
        best = 0
        for w in adj[u]:
            if state[w] == 1:
                raise CycleFound
            if state[w] == 0:
                dfs(w)
            best = max(best, memo[w])
        state[u] = 2
        memo[u] = best + 1

    class CycleFound(Exception):
        pass

    try:
        for v in range(n):
            if state[v] == 0:
                dfs(v)
    except CycleFound:
        return 0
    return max(memo) if n > 0 else 0
