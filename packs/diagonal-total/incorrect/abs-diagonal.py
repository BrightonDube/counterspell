def diagonal_total(grid):
    size = min(len(grid), len(grid[0]) if grid else 0)
    total = 0
    for i in range(size):
        total += abs(grid[i][i])
    return total
