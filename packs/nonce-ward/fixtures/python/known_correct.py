def nonce_ward(msgs):
    high = {}
    replays = 0
    for s, n in msgs:
        if n <= high.get(s, -1):
            replays += 1
        else:
            high[s] = n
    return replays
