def cover_pick(n, masks):
    full = (1 << n) - 1
    covered, picks = 0, 0
    while covered != full:
        best_gain, best_i = 0, -1
        for i, m in enumerate(masks):
            gain = bin(m & full & ~covered).count("1")
            if gain > best_gain:
                best_gain, best_i = gain, i
        if best_i < 0:
            return picks
        covered |= masks[best_i] & full
        picks += 1
    return picks
