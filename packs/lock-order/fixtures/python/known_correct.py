def lock_order(wards, lens):
    seen = set()
    at = 0
    for count in lens:
        row = wards[at:at + count]
        at += count
        for i in range(len(row)):
            for j in range(i + 1, len(row)):
                a, b = row[i], row[j]
                if a != b and (b, a) in seen:
                    return 0
                seen.add((a, b))
    return 1
