def matching_totals(entries_a, entries_b):
    def totals(entries):
        sums = {}
        for key, amount in entries:
            sums[key] = sums.get(key, 0) + amount
        return sums

    a = totals(entries_a)
    b = totals(entries_b)
    return sum(1 for key in a if key in b and a[key] == b[key])
