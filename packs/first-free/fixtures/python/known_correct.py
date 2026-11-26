def first_free(spans):
    cov = set()
    for a, b in spans:
        cov.update(range(a, b + 1))
    i = 0
    while i in cov:
        i += 1
    return i
