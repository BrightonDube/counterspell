def lone_count(edges, n):
    touched = [False] * n
    for a, b in edges:
        if 0 <= a < n:
            touched[a] = True
        if 0 <= b < n:
            touched[b] = True
    return sum(1 for t in touched if not t)
