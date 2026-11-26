def alloc_rite(blocks, reqs):
    slots = list(blocks)
    granted = 0
    for r in reqs:
        for i, b in enumerate(slots):
            if b >= r:
                slots[i] = b - r
                granted += 1
                break
    return granted
