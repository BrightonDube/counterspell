def prefix_counts(marks, queries):
    return [marks[:k].count(v) for k, v in queries]
