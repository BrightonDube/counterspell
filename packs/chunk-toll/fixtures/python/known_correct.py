def chunk_toll(tokens, w, base):
    n = len(tokens)
    w = max(w, 1)
    dp = [0] + [10**18] * n
    for i in range(1, n + 1):
        for j in range(max(0, i - w), i):
            c = tokens[j:i]
            cost = dp[j] + base + (max(c) - min(c))
            if cost < dp[i]:
                dp[i] = cost
    return dp[n]
