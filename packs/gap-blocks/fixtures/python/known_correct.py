def block_count(spans):
    if not spans:
        return 0
    s = sorted(spans)
    blocks = 1
    end = s[0][1]
    for a, b in s[1:]:
        if a > end:
            blocks += 1
            end = b
        else:
            end = max(end, b)
    return blocks
