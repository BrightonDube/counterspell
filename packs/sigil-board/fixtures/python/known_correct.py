def sigil_count(n):
    def f(r, cols):
        if r == n:
            return 1
        total = 0
        for c in range(n):
            if c in cols:
                continue
            if any(abs(r - rr) == abs(c - cc) for rr, cc in enumerate(cols)):
                continue
            total += f(r + 1, cols + [c])
        return total
    return f(0, [])
