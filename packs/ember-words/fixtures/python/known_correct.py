def word_break(text, lexicon):
    words = [w for w in lexicon.split(",") if w]
    n = len(text)
    d = [0] * (n + 1)
    d[0] = 1
    for i in range(1, n + 1):
        for w in words:
            if i >= len(w) and d[i - len(w)] and text[i - len(w):i] == w:
                d[i] = 1
                break
    return d[n]
