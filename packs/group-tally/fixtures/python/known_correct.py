def group_tally(rows):
    tally = {}
    for k, v in rows:
        tally[k] = tally.get(k, 0) + v
    return max(tally.values()) if tally else 0
