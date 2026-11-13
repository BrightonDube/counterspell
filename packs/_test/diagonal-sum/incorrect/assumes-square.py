# Deliberately wrong. It loops over the row count only, so a grid with more rows than
# columns reads past the end of a row and raises IndexError.
def diagonal_sum(grid):
    return sum(grid[i][i] for i in range(len(grid)))
