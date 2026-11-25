def largest_sigil(markings):
    groups = {}
    for marking in markings:
        sigil = sum(int(digit) for digit in str(marking))
        groups[sigil] = groups.get(sigil, 0) + 1
    return max(groups.values()) if groups else 0
