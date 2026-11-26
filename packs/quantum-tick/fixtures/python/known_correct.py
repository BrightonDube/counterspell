def quantum_tick(jobs, q):
    order = sorted(range(len(jobs)), key=lambda i: (jobs[i][0], i))
    rem = {i: jobs[i][1] for i in order}
    ready = []
    i, t, total, done = 0, 0, 0, 0
    n = len(order)
    while done < n:
        while i < n and jobs[order[i]][0] <= t:
            ready.append(order[i]); i += 1
        if not ready:
            t = jobs[order[i]][0]
            continue
        j = ready.pop(0)
        run = min(q, rem[j])
        t += run
        rem[j] -= run
        while i < n and jobs[order[i]][0] <= t:
            ready.append(order[i]); i += 1
        if rem[j] > 0:
            ready.append(j)
        else:
            total += t
            done += 1
    return total
