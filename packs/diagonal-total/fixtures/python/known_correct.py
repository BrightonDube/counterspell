def diagonal_total(grid):
    edge = min(len(grid), len(grid[0]) if grid else 0)
    return sum(grid[row][row] for row in range(edge))
