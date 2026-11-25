def deepest_ember(inscription):
    depth = 0
    deepest = 0
    for rune in inscription:
        if rune == "(":
            depth += 1
            deepest = max(deepest, depth)
        elif depth > 0:
            depth -= 1
    return deepest
