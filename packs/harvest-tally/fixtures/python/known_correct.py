def cross_pairs(rows):
    total = 0
    ahead = sum(rows)
    for count in rows:
        ahead -= count
        total += count * ahead
    return total
