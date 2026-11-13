def longest_run(s):
    best, cur, prev = 0, 0, None
    for ch in s:
        cur = cur + 1 if ch == prev else 1
        best = max(best, cur)
        prev = ch
    return best
