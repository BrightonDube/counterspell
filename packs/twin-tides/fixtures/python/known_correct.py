def merge_tides(first, second):
    merged = []
    a, b = 0, 0
    while a < len(first) and b < len(second):
        if first[a] <= second[b]:
            merged.append(first[a])
            a += 1
        else:
            merged.append(second[b])
            b += 1
    merged.extend(first[a:])
    merged.extend(second[b:])
    return merged
