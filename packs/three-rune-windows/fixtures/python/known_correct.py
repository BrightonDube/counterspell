def count_varied(inscription):
    n = len(inscription)
    total = 0
    for left in range(n):
        missing = {"a", "b", "c"}
        for right in range(left, n):
            missing.discard(inscription[right])
            if not missing:
                total += n - right
                break
    return total
