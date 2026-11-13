def longest_run(text):
    best = 0
    current = 0
    previous = None
    for ch in text:
        if ch == previous:
            current += 1
        else:
            current = 1
            previous = ch
        if current > best:
            best = current
    return best
