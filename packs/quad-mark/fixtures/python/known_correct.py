def quad_mark(x, y):
    if x == 0 or y == 0:
        return 0
    if y > 0:
        return 1 if x > 0 else 2
    return 4 if x > 0 else 3
