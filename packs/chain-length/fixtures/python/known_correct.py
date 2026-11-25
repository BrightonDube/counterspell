def chain_length(nxt):
    i, count = (0, 0) if nxt else (-1, 0)
    while i != -1:
        count += 1
        i = nxt[i]
    return count
