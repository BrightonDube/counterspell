def toll_stairs(costs):
    n = len(costs)
    if n <= 1:
        return costs[0] if n == 1 else 0
    d = [0] * (n + 1)
    for i in range(2, n + 1):
        d[i] = min(d[i - 1] + costs[i - 1], d[i - 2] + costs[i - 2])
    return d[n]
