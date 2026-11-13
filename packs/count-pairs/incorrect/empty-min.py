def count_pairs(values, target):
    # Early exit: no pair can add up to less than twice the smallest value.
    if target < 2 * min(values):
        return 0
    count = 0
    for i in range(len(values)):
        for j in range(i + 1, len(values)):
            if values[i] + values[j] == target:
                count += 1
    return count
