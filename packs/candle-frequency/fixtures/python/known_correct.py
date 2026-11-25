def top_burn(sizes):
    counts = {}
    for size in sizes:
        counts[size] = counts.get(size, 0) + 1
    best = 0
    best_count = 0
    for size, count in counts.items():
        if count > best_count or (count == best_count and size < best):
            best, best_count = size, count
    return best
