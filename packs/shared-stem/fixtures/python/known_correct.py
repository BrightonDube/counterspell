def common_stem(words):
    entries = [w for w in words.split(",") if w]
    if not entries:
        return 0
    stem = entries[0]
    for w in entries[1:]:
        while not w.startswith(stem):
            stem = stem[:-1]
    return len(stem)
