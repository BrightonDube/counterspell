def nth_tail(marks, nxt, n):
    if not nxt:
        return -1
    lead = trail = 0
    for _ in range(n):
        if lead == -1:
            return -1
        lead = nxt[lead]
    if lead == -1:
        return -1
    while nxt[lead] != -1:
        lead = nxt[lead]
        trail = nxt[trail]
    return marks[trail]
