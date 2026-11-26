def median_x2(marks):
    m = sorted(marks)
    n = len(m)
    if n == 0:
        return 0
    if n % 2:
        return 2 * m[n // 2]
    return m[n // 2 - 1] + m[n // 2]
