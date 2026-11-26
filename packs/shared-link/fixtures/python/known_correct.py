def shared_link(nxt, ha, hb):
    seen, i = set(), ha
    while i != -1:
        seen.add(i)
        i = nxt[i]
    j = hb
    while j != -1 and j not in seen:
        j = nxt[j]
    return j
