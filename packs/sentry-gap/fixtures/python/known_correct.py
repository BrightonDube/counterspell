def sentry_ways(cells, sentries, gap):
    def f(c, s):
        if s == 0:
            return 1
        if c <= 0:
            return 0
        return f(c - gap - 1, s - 1) + f(c - 1, s)
    return f(cells, sentries)
