def loop_flag(nxt):
    seen, i = set(), (0 if nxt else -1)
    while i != -1 and i not in seen:
        seen.add(i)
        i = nxt[i]
    return 1 if i != -1 else 0
