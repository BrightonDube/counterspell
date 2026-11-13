def longest_run(text):
    best = 1
    current = 1
    previous = text[0]
    for ch in text[1:]:
        if ch == previous:
            current += 1
            if current > best:
                best = current
        else:
            current = 1
            previous = ch
    return best
