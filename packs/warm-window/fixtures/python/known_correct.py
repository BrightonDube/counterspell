def warmest_window(readings, width):
    if len(readings) < width:
        return 0
    total = sum(readings[:width])
    best = total
    for i in range(width, len(readings)):
        total += readings[i] - readings[i - width]
        if total > best:
            best = total
    return best
