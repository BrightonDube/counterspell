def calm_haul(marks):
    take = skip = 0
    for m in marks:
        take, skip = skip + m, max(skip, take)
    return max(take, skip)
