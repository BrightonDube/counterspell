def longest_calm(chant):
    best = 0
    current = 0
    for rune in chant:
        if rune == "#":
            current = 0
        else:
            current += 1
            if current > best:
                best = current
    return best
