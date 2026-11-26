def norm_rite(v):
    s = sum(v)
    if s <= 0:
        return 0
    return 100 - sum((x * 100) // s for x in v)
