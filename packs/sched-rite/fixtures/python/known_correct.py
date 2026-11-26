def sched_rite(pairs):
    adj = {}
    for a, b in pairs:
        adj.setdefault(a, []).append(b)
    state = {}

    def visit(u):
        state[u] = 1
        for v in adj.get(u, []):
            s = state.get(v, 0)
            if s == 1:
                return True
            if s == 0 and visit(v):
                return True
        state[u] = 2
        return False

    for u in adj:
        if state.get(u, 0) == 0 and visit(u):
            return 0
    return 1
