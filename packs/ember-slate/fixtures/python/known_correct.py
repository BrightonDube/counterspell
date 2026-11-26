def slate_len(marks):
    if not marks:
        return 0
    d = [1] * len(marks)
    for i in range(len(marks)):
        for j in range(i):
            if marks[j] < marks[i]:
                d[i] = max(d[i], d[j] + 1)
    return max(d)
