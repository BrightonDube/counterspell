def decode_ways(glyphs):
    n = len(glyphs)
    if n == 0:
        return 1
    d = [0] * (n + 1)
    d[0] = 1
    for i in range(1, n + 1):
        if glyphs[i - 1] != "0":
            d[i] += d[i - 1]
        if i >= 2 and 10 <= int(glyphs[i - 2:i]) <= 26:
            d[i] += d[i - 2]
    return d[n]
