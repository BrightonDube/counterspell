# Deliberately wrong. It adds the absolute value of each diagonal cell instead of the cell.
def diagonal_sum(grid):
    rows = len(grid)
    cols = len(grid[0]) if rows else 0
    return sum(abs(grid[i][i]) for i in range(min(rows, cols)))
