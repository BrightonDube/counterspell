def ritual_spans(readings, quota):
    seen = {0: 1}
    total = 0
    spans = 0
    for reading in readings:
        total += reading
        spans += seen.get(total - quota, 0)
        seen[total] = seen.get(total, 0) + 1
    return spans
