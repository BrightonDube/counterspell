def loop_entry(nxt):
    seen, i = set(), (0 if nxt else -1)
    while i != -1 and i not in seen:
        seen.add(i)
        i = nxt[i]
    return i
