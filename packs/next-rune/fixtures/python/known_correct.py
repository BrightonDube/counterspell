def next_letters(words, prefix):
    nxts = set()
    k = len(prefix)
    for w in words.split(","):
        if w and w.startswith(prefix) and len(w) > k:
            nxts.add(w[k])
    return len(nxts)
