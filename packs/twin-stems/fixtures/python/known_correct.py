def prefix_pairs(words):
    ws = sorted(set(w for w in words.split(",") if w))
    ans = 0
    for i, a in enumerate(ws):
        for b in ws[i + 1:]:
            if a != b and b.startswith(a):
                ans += 1
    return ans
