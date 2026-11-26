def tail_mark(marks, nxt):
    i, last = (0, -1) if nxt else (-1, -1)
    while i != -1:
        last = i
        i = nxt[i]
    return marks[last] if last != -1 else -1
