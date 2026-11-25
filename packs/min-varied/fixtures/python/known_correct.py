def shortest_varied(inscription):
    counts = {"a": 0, "b": 0, "c": 0}
    held = 0
    left = 0
    best = len(inscription) + 1
    for right, rune in enumerate(inscription):
        if rune in counts:
            if counts[rune] == 0:
                held += 1
            counts[rune] += 1
        while held == 3:
            best = min(best, right - left + 1)
            edge = inscription[left]
            if edge in counts:
                counts[edge] -= 1
                if counts[edge] == 0:
                    held -= 1
            left += 1
    return 0 if best > len(inscription) else best
