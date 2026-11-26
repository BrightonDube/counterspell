def path_count(grid):
    rows = len(grid)
    cols = len(grid[0])
    def f(r, c):
        if r >= rows or c >= cols or grid[r][c] == 0:
            return 0
        if r == rows - 1 and c == cols - 1:
            return 1
        return f(r, c + 1) + f(r + 1, c)
    return f(0, 0)
