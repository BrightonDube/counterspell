def dot_sparse(a, b):
    da = {p: m for p, m in a}
    db = {p: m for p, m in b}
    return sum(da[p] * db[p] for p in da if p in db)
