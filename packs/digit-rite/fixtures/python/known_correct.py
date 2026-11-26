def digit_rite(n):
    orig, count = n, 0
    if n == 0:
        return 0
    while n:
        d = n % 10
        if d and orig % d == 0:
            count += 1
        n //= 10
    return count
