def cover_length(spans):
    cov = [False] * 101
    for a, b in spans:
        for i in range(a, b + 1):
            cov[i] = True
    return sum(cov)
