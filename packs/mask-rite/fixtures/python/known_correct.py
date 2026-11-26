def mask_rite(weights, cap):
    n = len(weights)
    count = 0
    for mask in range(1 << n):
        total = 0
        for i in range(n):
            if mask >> i & 1:
                total += weights[i]
        if total <= cap:
            count += 1
    return count
