def join_span(a, b):
    return sum(1 for x in a for y in b if x[0] == y[0] and x[1] < y[1])
