def _match(w, p):
    dp = [[False] * (len(p) + 1) for _ in range(len(w) + 1)]
    dp[0][0] = True
    for j in range(1, len(p) + 1):
        if p[j - 1] == "*":
            dp[0][j] = dp[0][j - 1]
    for i in range(1, len(w) + 1):
        for j in range(1, len(p) + 1):
            dp[i][j] = (
                p[j - 1] == "*" and (dp[i][j - 1] or dp[i - 1][j])
            ) or (
                p[j - 1] == w[i - 1] and dp[i - 1][j - 1]
            )
    return dp[len(w)][len(p)]


def count_matches(words, pattern):
    ws = set(w for w in words.split(",") if w)
    return sum(1 for w in ws if _match(w, pattern))
