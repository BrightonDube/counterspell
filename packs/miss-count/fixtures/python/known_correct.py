def miss_count(marks, c):
    slots = [-1] * c
    misses = 0
    for m in marks:
        line, tag = m % c, m // c
        if slots[line] != tag:
            misses += 1
            slots[line] = tag
    return misses
