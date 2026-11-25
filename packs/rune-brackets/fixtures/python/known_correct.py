def nested_runes(inscription):
    partners = {")": "(", "]": "[", "}": "{"}
    openers = []
    for rune in inscription:
        if rune in partners:
            if not openers or openers[-1] != partners[rune]:
                return 0
            openers.pop()
        else:
            openers.append(rune)
    return 1 if not openers else 0
