def gap_count(marks, cap):
    def go(i, left):
        if left < 0:
            return 0
        if i == len(marks):
            return 1
        return go(i + 1, left - marks[i]) + go(i + 1, left)
    return go(0, cap)
