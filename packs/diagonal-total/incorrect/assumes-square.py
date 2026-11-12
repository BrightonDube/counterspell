def diagonal_total(grid):
    total = 0
    for i in range(len(grid)):
        total += grid[i][i]
    return total
