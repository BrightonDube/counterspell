def frame_rite(pages, f):
    frame = []
    faults = 0
    n = len(pages)
    for i, p in enumerate(pages):
        if p in frame:
            continue
        faults += 1
        if len(frame) < f:
            frame.append(p)
        else:
            worst, worst_next = 0, -1
            for k, q in enumerate(frame):
                nxt = n + 1
                for j in range(i + 1, n):
                    if pages[j] == q:
                        nxt = j
                        break
                if nxt > worst_next:
                    worst, worst_next = k, nxt
            frame[worst] = p
    return faults
