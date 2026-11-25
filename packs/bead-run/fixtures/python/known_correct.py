def longest_bead(strand):
    best = 0
    run = 0
    previous = ""
    for bead in strand:
        run = run + 1 if bead == previous else 1
        if run > best:
            best = run
        previous = bead
    return best
