def stem_depth_sum(words):
    ws = set(w for w in words.split(",") if w)
    return sum(
        max((len(o) for o in ws if o != w and w.startswith(o)), default=0)
        for w in ws
    )
