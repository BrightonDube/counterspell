def braid_ways(text, first, second):
    n, m = len(first), len(second)
    if len(text) != n + m:
        return 0
    d = [[0] * (m + 1) for _ in range(n + 1)]
    d[0][0] = 1
    for i in range(n + 1):
        for j in range(m + 1):
            if i > 0 and first[i - 1] == text[i + j - 1]:
                d[i][j] |= d[i - 1][j]
            if j > 0 and second[j - 1] == text[i + j - 1]:
                d[i][j] |= d[i][j - 1]
    return d[n][m]
