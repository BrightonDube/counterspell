def partition_count(marks, ways):
    total = sum(marks)
    if total % ways or len(marks) < ways:
        return 0
    target = total // ways
    sums = [0] * ways
    counts = [0] * ways
    n = len(marks)

    def go(i):
        if i == n:
            return 1 if all(c > 0 for c in counts) else 0
        placed = 0
        for g in range(ways):
            if counts[g] == 0 and g > 0 and counts[g - 1] == 0:
                break
            if sums[g] + marks[i] > target:
                continue
            sums[g] += marks[i]
            counts[g] += 1
            placed += go(i + 1)
            sums[g] -= marks[i]
            counts[g] -= 1
            if counts[g] == 0:
                break
        return placed

    return go(0)
