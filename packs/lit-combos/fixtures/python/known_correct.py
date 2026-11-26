def count_mixes(coins, target):
    d = [0] * (target + 1)
    d[0] = 1
    for c in coins:
        for i in range(c, target + 1):
            d[i] += d[i - c]
    return d[target]
