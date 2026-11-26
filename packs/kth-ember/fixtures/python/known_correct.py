def kth_mark(marks, k):
    marks = sorted(marks)
    return marks[k - 1] if 0 < k <= len(marks) else -1
