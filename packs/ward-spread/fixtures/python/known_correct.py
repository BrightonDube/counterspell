def spread_ways(cells, guards):
    if guards == 0:
        return 1
    if cells <= 0:
        return 0
    return spread_ways(cells - 2, guards - 1) + spread_ways(cells - 1, guards)
