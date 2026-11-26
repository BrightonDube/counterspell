def serial_check(events):
    done = set()
    prev = None
    for t in events:
        if t in done:
            return 0
        if t != prev:
            if prev is not None:
                done.add(prev)
            prev = t
    return 1
