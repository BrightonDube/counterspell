def burst_rite(jobs):
    order = sorted(range(len(jobs)), key=lambda i: (jobs[i][0], i))
    clock, total = 0, 0
    for i in order:
        a, b = jobs[i]
        start = max(clock, a)
        total += start - a
        clock = start + b
    return total
