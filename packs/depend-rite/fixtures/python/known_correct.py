def depend_rite(rows):
    seen = {}
    for x, y in rows:
        if x in seen and seen[x] != y:
            return 0
        seen[x] = y
    return 1
