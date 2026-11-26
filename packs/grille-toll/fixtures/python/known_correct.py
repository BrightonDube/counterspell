def toll_grid(grid):
    h = len(grid)
    w = len(grid[0])
    big = 10**18
    d = [[big] * w for _ in range(h)]
    for i in range(h):
        for j in range(w):
            if grid[i][j] != 0:
                continue
            if i == 0 and j == 0:
                d[i][j] = 1
            else:
                best = min(d[i - 1][j] if i > 0 else big,
                           d[i][j - 1] if j > 0 else big)
                if best < big:
                    d[i][j] = best + 1
    return d[-1][-1] if d[-1][-1] < big else -1
