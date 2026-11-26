def wrap_rite(marks, b):
    ring = []
    evictions = 0
    for m in marks:
        if m not in ring:
            if len(ring) == b:
                ring.pop(0)
                evictions += 1
            ring.append(m)
    return evictions
