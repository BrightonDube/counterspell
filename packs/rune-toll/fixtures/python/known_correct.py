def toll_sum(edges, weights, n):
    total = 0
    for k in range(min(len(edges), len(weights))):
        total += weights[k]
    return total
