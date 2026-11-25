def longest_thrifty(pane):
    counts = {}
    left = 0
    best = 0
    for right, rune in enumerate(pane):
        counts[rune] = counts.get(rune, 0) + 1
        while len(counts) > 2:
            edge = pane[left]
            counts[edge] -= 1
            if counts[edge] == 0:
                del counts[edge]
            left += 1
        best = max(best, right - left + 1)
    return best
