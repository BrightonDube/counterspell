def split_count(marks):
    total = sum(marks)
    if total % 2:
        return 0
    half = total // 2

    def go(i, need):
        if need == 0:
            return 1
        if i == len(marks):
            return 0
        return go(i + 1, need - marks[i]) + go(i + 1, need)

    subsets = go(0, half)
    if half == 0:
        # every mark is zero, so all subsets tie; drop the empty and the full.
        return max(0, 2 ** len(marks) - 2) // 2
    return subsets // 2
