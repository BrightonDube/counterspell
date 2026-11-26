def shelf_max(grid):
    h = len(grid)
    w = len(grid[0])
    d = [[0] * w for _ in range(h)]
    for i in range(h):
        for j in range(w):
            if i == 0 and j == 0:
                d[i][j] = grid[0][0]
            else:
                best = max(d[i - 1][j] if i > 0 else -10**18,
                           d[i][j - 1] if j > 0 else -10**18)
                d[i][j] = best + grid[i][j]
    return d[-1][-1]
