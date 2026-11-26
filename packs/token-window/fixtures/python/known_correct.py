def token_window(tokens, w, mark):
    w = min(w, len(tokens))
    if w <= 0:
        return 0
    cur = sum(1 for t in tokens[:w] if t == mark)
    best = cur
    for i in range(w, len(tokens)):
        cur += (tokens[i] == mark) - (tokens[i - w] == mark)
        if cur > best:
            best = cur
    return best
