def shrunk_count(words):
    ws = set(w for w in words.split(",") if w)
    return sum(
        1
        for w in ws
        if not any(o != w and o.startswith(w) for o in ws)
    )
