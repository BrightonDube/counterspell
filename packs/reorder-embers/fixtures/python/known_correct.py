def reorder_embers(marks, nxt):
    order, i = [], (0 if nxt else -1)
    while i != -1:
        order.append(marks[i])
        i = nxt[i]
    half = (len(order) + 1) // 2
    front, back = order[:half], order[half:][::-1]
    out = []
    for k in range(len(front)):
        out.append(front[k])
        if k < len(back):
            out.append(back[k])
    return out
