def tally_frames(m):
    return sum(m[i][i] for i in range(len(m)) if i < len(m[i]))
