def can_segment(words, s):
    ws = set(w for w in words.split(",") if w)
    ok = [True] + [False] * len(s)
    for i in range(len(s) + 1):
        if not ok[i]:
            continue
        for w in ws:
            if s.startswith(w, i):
                ok[i + len(w)] = True
    return 1 if ok[-1] else 0
