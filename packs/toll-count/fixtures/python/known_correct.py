def toll_count(edges, weights, t):
    count = 0
    for k in range(min(len(edges), len(weights))):
        if weights[k] <= t:
            count += 1
    return count
