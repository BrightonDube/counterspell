# Deliberately wrong. It starts from s[0], which raises IndexError on an empty string.
def longest_run(s):
    best = cur = 1
    prev = s[0]
    for ch in s[1:]:
        cur = cur + 1 if ch == prev else 1
        best = max(best, cur)
        prev = ch
    return best
