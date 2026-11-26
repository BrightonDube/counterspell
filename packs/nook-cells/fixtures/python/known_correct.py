def nook_ways(grid):
    h = len(grid)
    w = len(grid[0])
    d = [[0] * w for _ in range(h)]
    for i in range(h):
        for j in range(w):
            if grid[i][j] == 0:
                continue
            if i == 0 and j == 0:
                d[i][j] = 1
            else:
                d[i][j] = (d[i - 1][j] if i > 0 else 0) + (d[i][j - 1] if j > 0 else 0)
    return d[-1][-1]
