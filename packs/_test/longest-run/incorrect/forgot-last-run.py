# Deliberately wrong. best is updated only when a run ends, so a run that reaches
# the end of the string is never counted.
def longest_run(s):
    best, cur, prev = 0, 0, None
    for ch in s:
        if ch == prev:
            cur += 1
        else:
            best = max(best, cur)
            cur = 1
        prev = ch
    return best
