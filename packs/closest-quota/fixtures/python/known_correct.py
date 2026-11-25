def nearest_pair(marks, quota):
    if len(marks) < 2:
        return 0
    left, right = 0, len(marks) - 1
    best = None
    while left < right:
        total = marks[left] + marks[right]
        if best is None or abs(total - quota) < abs(best - quota) or (
            abs(total - quota) == abs(best - quota) and total < best
        ):
            best = total
        if total < quota:
            left += 1
        else:
            right -= 1
    return best
