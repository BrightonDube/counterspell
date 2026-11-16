def count_pairs(values, target):
    pairs = 0
    for left in range(len(values)):
        for right in range(left + 1, len(values)):
            if values[left] + values[right] == target:
                pairs += 1
    return pairs
