def diagonal_sum(grid):
    rows = len(grid)
    cols = len(grid[0]) if rows else 0
    return sum(grid[i][i] for i in range(min(rows, cols)))
