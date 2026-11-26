def stale_count(seen, queued):
    return sum(1 for a, b in zip(seen, queued) if a != b)
