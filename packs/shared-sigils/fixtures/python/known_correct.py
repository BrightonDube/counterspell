def shared_marks(first, second):
    shared = []
    a, b = 0, 0
    while a < len(first) and b < len(second):
        if first[a] < second[b]:
            a += 1
        elif second[b] < first[a]:
            b += 1
        else:
            mark = first[a]
            if not shared or shared[-1] != mark:
                shared.append(mark)
            a += 1
            b += 1
    return shared
