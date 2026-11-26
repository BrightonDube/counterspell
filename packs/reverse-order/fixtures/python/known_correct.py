def reverse_order(marks, nxt):
    out, i = [], (0 if nxt else -1)
    while i != -1:
        out.append(marks[i])
        i = nxt[i]
    return out[::-1]
