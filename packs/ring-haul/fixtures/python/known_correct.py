def ring_haul(marks):
    n = len(marks)
    if n <= 1:
        return marks[0] if n == 1 else 0

    def line(ms):
        take = skip = 0
        for m in ms:
            take, skip = skip + m, max(skip, take)
        return max(take, skip)

    return max(line(marks[1:]), line(marks[:-1]))
