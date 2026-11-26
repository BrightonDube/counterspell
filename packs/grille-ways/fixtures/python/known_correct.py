def grille_ways(rows, cols):
    if rows <= 0 or cols <= 0:
        return 0
    d = [[0] * cols for _ in range(rows)]
    for i in range(rows):
        for j in range(cols):
            d[i][j] = 1 if i == 0 or j == 0 else d[i - 1][j] + d[i][j - 1]
    return d[-1][-1]
