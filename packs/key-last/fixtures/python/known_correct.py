def key_last(rows, k):
    hit = -1
    for key, v in rows:
        if key == k:
            hit = v
    return hit
