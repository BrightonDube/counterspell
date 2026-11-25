def longest_harvest(marks, quota):
    left = 0
    total = 0
    best = 0
    for right, mark in enumerate(marks):
        total += mark
        while total > quota:
            total -= marks[left]
            left += 1
        best = max(best, right - left + 1)
    return best
